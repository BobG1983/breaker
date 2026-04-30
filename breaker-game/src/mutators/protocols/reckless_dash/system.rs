//! `RecklessDash` protocol — risky-catch damage boost with bolt-loss penalty
//! doubling on dash transitions.
//!
//! Design doc: `docs/design/protocols/reckless_dash.md`.
//!
//! Owns the `RecklessDashConfig` resource (per-run tuning), the
//! `RiskyDamageBoost` per-bolt component, the `OriginalBoltLossBehavior`
//! per-breaker overlay component, the builder-produced
//! `"protocol:reckless_dash"` source tag, the `activate` / `wire` dispatch
//! entry points, and the four runtime systems
//! (`reckless_dash_on_bump`, `reckless_dash_amplify_damage`,
//! `reckless_dash_on_dash_transition`, `reckless_dash_cleanup_node`).

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE, sets::BoltSystems},
    breaker::{
        components::{
            BoltLossBehavior, DashDuration, DashState, DashStateTimer, PreviousDashState,
        },
        sets::BreakerSystems,
    },
    mutators::protocols::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
        systems::ProtocolGate,
    },
    prelude::*,
};

// ── RecklessDashConfig ─────────────────────────────────────────────────────

/// Per-run Reckless Dash tuning extracted from
/// `ProtocolTuning::RecklessDash` at activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct RecklessDashConfig {
    /// Dash-progress threshold above which a bump is considered "risky".
    /// Semantics: `progress > risky_zone_start` is risky; exact equality is
    /// NOT risky (strict inequality).
    pub(crate) risky_zone_start:  f32,
    /// Multiplier applied to `BoltBaseDamage` on a risky catch.
    pub(crate) damage_multiplier: f32,
    /// When true, `reckless_dash_on_dash_transition` doubles the breaker's
    /// `BoltLossBehavior` for the duration of every dash.
    pub(crate) double_penalty:    bool,
}

// ── RiskyDamageBoost ───────────────────────────────────────────────────────

/// Per-bolt single-shot amplified-damage marker inserted by
/// `reckless_dash_on_bump` on a risky catch and consumed by
/// `reckless_dash_amplify_damage` on the bolt's next cell impact.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RiskyDamageBoost {
    /// Multiplier carried by this boost (copied from
    /// `RecklessDashConfig::damage_multiplier` at insert time).
    pub multiplier: f32,
}

// ── OriginalBoltLossBehavior ───────────────────────────────────────────────

/// Per-breaker overlay that stores the unmodified `BoltLossBehavior` from
/// before `reckless_dash_on_dash_transition` began doubling it. Inserted on
/// the `Idle → Dashing` transition; removed on `Dashing → {anything else}`.
/// Allows restoration of the original behavior when the dash ends.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct OriginalBoltLossBehavior(
    /// The original, undoubled `BoltLossBehavior` captured at `Idle → Dashing` entry.
    pub BoltLossBehavior,
);

// ── reckless_dash_on_dash_transition ──────────────────────────────────────

/// Query alias for `reckless_dash_on_dash_transition` — extracted to keep the
/// system signature under clippy's `type_complexity` threshold.
type BreakerDashQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static DashState,
        &'static PreviousDashState,
        &'static mut BoltLossBehavior,
        Option<&'static OriginalBoltLossBehavior>,
    ),
    (With<Breaker>, Changed<DashState>),
>;

/// Mutates `BoltLossBehavior` on dash state transitions.
///
/// - `{not Dashing} → Dashing`: saves the live `BoltLossBehavior` as
///   `OriginalBoltLossBehavior`, then doubles the live component
///   (`LifeLoss(n) → LifeLoss(n.saturating_mul(2))`,
///   `TimeLoss(d) → TimeLoss(d * 2.0)`, `None → None`).
/// - `Dashing → {anything else}`: restores `BoltLossBehavior` from the
///   `OriginalBoltLossBehavior` overlay (when present) and removes the
///   overlay component. Safe no-op when the overlay is absent.
///
/// Skips (early return) when:
/// - `RecklessDashConfig` is absent.
/// - `config.double_penalty == false`.
///
/// Run conditions (registered in `wire`): gated on
/// `protocol_active(ProtocolKind::RecklessDash)` AND `in_state(NodeState::Playing)`.
///
/// Transition detection compares `PreviousDashState` against the current
/// `DashState` rather than relying on `Changed<DashState>` alone — the
/// `Changed` filter guards iteration cost (it's the query filter), but
/// the actual enter/exit logic uses the previous-state snapshot to
/// distinguish same-value writes (e.g. `Idle → Idle`) from real
/// transitions.
pub(crate) fn reckless_dash_on_dash_transition(
    config: Option<Res<RecklessDashConfig>>,
    mut breakers: BreakerDashQuery,
    mut commands: Commands,
) {
    let Some(config) = config else {
        return;
    };
    if !config.double_penalty {
        return;
    }
    for (entity, current_state, previous_dash_state, mut current_behavior, overlay_opt) in
        &mut breakers
    {
        let was_dashing = matches!(previous_dash_state.0, DashState::Dashing);
        let now_dashing = matches!(*current_state, DashState::Dashing);
        if !was_dashing && now_dashing {
            // Read BEFORE mutating so the saved value is the unmodified one.
            let original = *current_behavior;
            commands
                .entity(entity)
                .insert(OriginalBoltLossBehavior(original));
            // (Site A) cross-domain write exception: reckless_dash uniquely observes Idle→Dashing
            // via PreviousDashState (1); simple enum-value mutation, not a state-machine
            // transition (2); gated by run_if(protocol_active) + run_if(in_state(Playing))
            // + Changed<DashState> query filter (3); message-based alternative would need
            // a new breaker consumer system solely for this one path (4).
            *current_behavior = double_behavior(original);
        } else if was_dashing
            && !now_dashing
            && let Some(overlay) = overlay_opt
        {
            // cross-domain write exception: same justification as Site A above (1-4);
            // restore is the paired inverse of the doubling write.
            *current_behavior = overlay.0;
            commands.entity(entity).remove::<OriginalBoltLossBehavior>();
        }
        // Neither enter nor exit (e.g., Idle → Idle same-value write,
        // Dashing → Dashing) is a no-op.
    }
}

/// Doubles the penalty carried by a `BoltLossBehavior`.
///
/// - `LifeLoss(n)` → `LifeLoss(n.saturating_mul(2))` (no panic on overflow).
/// - `TimeLoss(d)` → `TimeLoss(d * 2.0)` (raw float multiplication, no clamp).
///   NaN input produces NaN output; callers must ensure values are finite.
/// - `None` → `None` (uniform handling).
fn double_behavior(behavior: BoltLossBehavior) -> BoltLossBehavior {
    match behavior {
        BoltLossBehavior::LifeLoss(n) => BoltLossBehavior::LifeLoss(n.saturating_mul(2)),
        BoltLossBehavior::TimeLoss(d) => BoltLossBehavior::TimeLoss(d * 2.0),
        BoltLossBehavior::None => BoltLossBehavior::None,
    }
}

// ── activate ───────────────────────────────────────────────────────────────

/// Inserts `RecklessDashConfig` from `ProtocolTuning::RecklessDash`. Warns
/// and no-ops on a non-`RecklessDash` tuning variant, leaving any existing
/// `RecklessDashConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::RecklessDash {
        risky_zone_start,
        damage_multiplier,
        double_penalty,
    } = *tuning
    else {
        warn!("reckless_dash::activate called with non-RecklessDash tuning");
        return;
    };
    commands.insert_resource(RecklessDashConfig {
        risky_zone_start,
        damage_multiplier,
        double_penalty,
    });
}

// ── wire ───────────────────────────────────────────────────────────────

/// Registers Reckless Dash's runtime systems with the correct schedules,
/// run-ifs, and ordering.
///
/// `FixedUpdate` reader systems — intentionally ungated at the registration
/// level. Each system enforces the `ActiveProtocols` / `NodeState::Playing`
/// gate in-body via an immediate `reader.clear()` + return when inactive,
/// draining the buffer every tick so buffered messages cannot leak into a
/// later frame where the protocol activates:
/// - `reckless_dash_on_bump` — after `BreakerSystems::GradeBump`.
/// - `reckless_dash_amplify_damage` — after `BoltSystems::CellCollision`.
///
/// `FixedUpdate` non-reader system — gated externally via `run_if` because
/// it has no message reader to drain:
/// - `reckless_dash_on_dash_transition` —
///   `.after(BreakerSystems::UpdateState)
///    .before(BreakerSystems::UpdatePreviousState)
///    .before(BreakerSystems::HandleBoltLost)`,
///   gated on `protocol_active(RecklessDash)` AND
///   `in_state(NodeState::Playing)`.
///
/// `OnExit(NodeState::Playing)` cleanup system:
/// - `reckless_dash_cleanup_node` — restores `BoltLossBehavior` from
///   `OriginalBoltLossBehavior` on every breaker that exits `Playing` while
///   still dashing. Runs unconditionally (no run-if gate).
pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            reckless_dash_on_bump.after(BreakerSystems::GradeBump),
            reckless_dash_amplify_damage
                .after(BoltSystems::CellCollision)
                .in_set(DmgSystems::EmitDamage),
            reckless_dash_on_dash_transition
                .after(BreakerSystems::UpdateState)
                .before(BreakerSystems::UpdatePreviousState)
                .before(BreakerSystems::HandleBoltLost)
                .run_if(protocol_active(ProtocolKind::RecklessDash))
                .run_if(in_state(NodeState::Playing)),
        ),
    );
    app.add_systems(OnExit(NodeState::Playing), reckless_dash_cleanup_node);
}

/// Query alias for [`reckless_dash_cleanup_node`] — extracts breakers that
/// exited `NodeState::Playing` while carrying an `OriginalBoltLossBehavior`
/// overlay (i.e., while still `DashState::Dashing`).
type BreakerOverlayQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut BoltLossBehavior,
        &'static OriginalBoltLossBehavior,
    ),
    With<Breaker>,
>;

/// Runs on `OnExit(NodeState::Playing)`. For every breaker that still carries
/// an `OriginalBoltLossBehavior` overlay (i.e., exited `Playing` while
/// `DashState::Dashing`), restores `BoltLossBehavior` to the saved original
/// and removes the overlay.
///
/// The query's `&OriginalBoltLossBehavior` filter naturally skips breakers
/// without the overlay — idle breakers are untouched.
pub(crate) fn reckless_dash_cleanup_node(
    mut breakers: BreakerOverlayQuery,
    mut commands: Commands,
) {
    for (entity, mut current_behavior, overlay) in &mut breakers {
        // cross-domain write exception: same justification as the transition-system
        // restore (1-4); this cleanup runs only on OnExit(NodeState::Playing) so the
        // run-condition gate is structural, not a run_if (3).
        *current_behavior = overlay.0;
        commands.entity(entity).remove::<OriginalBoltLossBehavior>();
    }
}

// ── Systems ────────────────────────────────────────────────────────────────

/// Consumes `BumpPerformed` messages. When the breaker is `DashState::Dashing`
/// and the dash progress fraction `(duration - remaining) / duration` is
/// strictly greater than `config.risky_zone_start`, inserts
/// `RiskyDamageBoost { multiplier: config.damage_multiplier }` on the bolt.
///
/// Skips (`continue`) when:
/// - `BumpPerformed.bolt` is `None` (spectator bump).
/// - The breaker lookup fails (despawned or missing components).
/// - The breaker is not `Dashing`.
/// - `DashDuration(0.0)` — avoids divide-by-zero on degenerate dashes.
/// - `progress <= config.risky_zone_start` (strict inequality).
///
/// Harness-safe: if `RecklessDashConfig` is absent, clears the reader and
/// returns so buffered messages do not leak into a later frame that does have
/// the resource.
pub(crate) fn reckless_dash_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<RecklessDashConfig>>,
    gate: ProtocolGate,
    breakers: Query<(&DashState, &DashStateTimer, &DashDuration), With<Breaker>>,
    mut commands: Commands,
) {
    if gate.is_closed_for(ProtocolKind::RecklessDash) {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok((state, timer, duration)) = breakers.get(msg.breaker) else {
            continue;
        };
        if *state != DashState::Dashing {
            continue;
        }
        if duration.0 <= f32::EPSILON {
            continue;
        }
        // `timer.remaining` is in seconds, clamped to `[0.0, duration.0]` by
        // the breaker dash systems; `progress` lies in `[0.0, 1.0]` where
        // 0.0 is "dash just started" and 1.0 is "dash just finished".
        let progress = (duration.0 - timer.remaining) / duration.0;
        if progress > config.risky_zone_start {
            // `get_entity` returns `Err` for a despawned bolt — silently skip
            // rather than panicking at command-flush time.
            if let Ok(mut entity) = commands.get_entity(bolt) {
                entity.insert(RiskyDamageBoost {
                    multiplier: config.damage_multiplier,
                });
            }
        }
    }
}

/// Consumes `BoltImpactCell` messages. On a bolt carrying `RiskyDamageBoost`,
/// emits an amplified `DamageDealt<Cell>` with
/// `amount = base_damage * boost.multiplier` and
/// `source = Some(SourceId::protocol(RecklessDash).build())` — gated on
/// `amount > 0.0`. Removes the `RiskyDamageBoost` from the bolt unconditionally
/// (single-shot, even when emission is gated off by the `amount > 0.0` guard).
///
/// Non-boosted bolts are ignored entirely — this system does NOT emit
/// baseline damage (that is `bolt_cell_collision`'s responsibility).
///
/// Pierce guard: `bolt_cell_collision` can emit multiple `BoltImpactCell`
/// messages for the same bolt in a single frame (pierce-through). The
/// `RiskyDamageBoost` removal is deferred via `commands`, so subsequent
/// iterations in the same invocation would otherwise still see the boost.
/// `amplified_this_frame` tracks already-amplified bolts within THIS
/// invocation so only ONE amplified emit is produced per boost.
///
/// Harness-safe: if `RecklessDashConfig` is absent, clears the reader and
/// returns so buffered messages do not leak into a later frame that does have
/// the resource. Boost is NOT consumed in this path — only the early-return
/// drain happens.
pub(crate) fn reckless_dash_amplify_damage(
    mut reader: MessageReader<BoltImpactCell>,
    config: Option<Res<RecklessDashConfig>>,
    gate: ProtocolGate,
    bolts: Query<(&RiskyDamageBoost, Option<&BoltBaseDamage>)>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    if gate.is_closed_for(ProtocolKind::RecklessDash) {
        reader.clear();
        return;
    }
    if config.is_none() {
        reader.clear();
        return;
    }
    // `Vec` (not `HashSet`) because cardinality is almost always 0 or 1: a
    // single-bolt game with one `BoltImpactCell` message per frame. Linear
    // `contains` beats hash overhead at this scale, and the pre-sized
    // capacity avoids allocation on the common path.
    let mut amplified_this_frame: Vec<Entity> = Vec::with_capacity(1);
    for msg in reader.read() {
        if amplified_this_frame.contains(&msg.bolt) {
            continue;
        }
        let Ok((boost, base_opt)) = bolts.get(msg.bolt) else {
            continue;
        };
        let base_damage = base_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |b| b.0);
        let amount = base_damage * boost.multiplier;
        if amount > 0.0 {
            damage_writer.write(DamageDealt::<Cell> {
                dealer: Some(msg.bolt),
                attributed_to: None,
                target: msg.cell,
                amount,
                source: Some(SourceId::protocol(ProtocolKind::RecklessDash).build()),
                _marker: PhantomData,
            });
        }
        // Unconditionally consume the boost — single-shot regardless of the
        // `amount > 0.0` emission gate.
        if let Ok(mut entity) = commands.get_entity(msg.bolt) {
            entity.remove::<RiskyDamageBoost>();
        }
        amplified_this_frame.push(msg.bolt);
    }
}
