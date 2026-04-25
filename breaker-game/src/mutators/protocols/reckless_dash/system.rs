//! `RecklessDash` protocol — risky-catch damage boost with double-penalty on
//! bolt loss.
//!
//! Design doc: `docs/design/protocols/reckless_dash.md`.
//!
//! Owns the `RecklessDashConfig` resource (per-run tuning), the
//! `RiskyDamageBoost` per-bolt component, the `RecklessDashDoubledBolts`
//! per-node tracking resource, the builder-produced `"protocol:reckless_dash"` source tag,
//! the `activate` / `register` dispatch entry points, and the four runtime
//! systems (`reckless_dash_on_bump`, `reckless_dash_amplify_damage`,
//! `reckless_dash_double_penalty`, `reckless_dash_cleanup_node`).

use std::{collections::HashSet, marker::PhantomData};

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE, sets::BoltSystems},
    breaker::{
        components::{DashDuration, DashState, DashStateTimer},
        sets::BreakerSystems,
    },
    effect_v3::EffectV3Systems,
    mutators::protocols::{
        definition::{ProtocolKind, ProtocolTuning},
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
    /// When true, every `BoltLost` that occurs while the breaker is Dashing
    /// is duplicated into a second `BoltLost`.
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

// ── RecklessDashDoubledBolts ───────────────────────────────────────────────

/// Per-node set of bolt entities whose `BoltLost` has already been duplicated
/// by `reckless_dash_double_penalty`. Used as the infinite-loop guard so a
/// duplicated `BoltLost` is not itself re-duplicated. Cleared on
/// `OnExit(NodeState::Playing)` by `reckless_dash_cleanup_node`.
///
/// Initialised by the `ProtocolPlugin` (matches Greed / Siphon / Fission
/// convention) — NOT by `register`.
#[derive(Resource, Debug, Default)]
pub struct RecklessDashDoubledBolts(pub HashSet<Entity>);

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

// ── register ───────────────────────────────────────────────────────────────

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
/// - `reckless_dash_double_penalty` — after `BoltSystems::BoltLost`.
///
/// `OnExit(NodeState::Playing)` (no run-if):
/// - `reckless_dash_cleanup_node` — clears the `RecklessDashDoubledBolts`
///   anti-feedback set.
///
/// NOTE: `register` does NOT call
/// `init_resource::<RecklessDashDoubledBolts>()`. The resource is initialised
/// by `ProtocolPlugin::build`, mirroring the Greed / Siphon / Fission
/// precedent where the plugin owns all protocol resource initialisation.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            reckless_dash_on_bump.after(BreakerSystems::GradeBump),
            reckless_dash_amplify_damage
                .after(BoltSystems::CellCollision)
                .before(EffectV3Systems::Bridge),
            reckless_dash_double_penalty.after(BoltSystems::BoltLost),
        ),
    )
    .add_systems(OnExit(NodeState::Playing), reckless_dash_cleanup_node);
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

/// Consumes `BoltLost` messages. On a `BoltLost` whose breaker is
/// `DashState::Dashing` and `config.double_penalty == true`, emits a second
/// `BoltLost` carrying the same `bolt` + `breaker` fields. The anti-feedback
/// set `RecklessDashDoubledBolts` prevents the same bolt from being doubled
/// more than once within a single node.
///
/// Skips (`continue`) per-message when:
/// - The breaker lookup fails (despawned or missing `DashState`).
/// - The breaker is not `Dashing`.
/// - The bolt is already in `RecklessDashDoubledBolts` (already doubled).
///
/// Harness-safe:
/// - If `RecklessDashConfig` is absent → drain the reader and return.
/// - If `config.double_penalty == false` → drain the reader and return
///   (disabled means no-op, not deferred).
pub(crate) fn reckless_dash_double_penalty(
    mut reader: MessageReader<BoltLost>,
    config: Option<Res<RecklessDashConfig>>,
    gate: ProtocolGate,
    breakers: Query<&DashState, With<Breaker>>,
    mut doubled: ResMut<RecklessDashDoubledBolts>,
    mut already_doubled_ever: Local<HashSet<Entity>>,
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
    if !config.double_penalty {
        reader.clear();
        return;
    }
    for msg in reader.read() {
        let Ok(state) = breakers.get(msg.breaker) else {
            continue;
        };
        if *state != DashState::Dashing {
            continue;
        }
        // Two-level anti-feedback guard — division of responsibility:
        //
        // - `already_doubled_ever` (system-local, NEVER cleared): catches the
        //   one failure mode the observable set can't — after an `OnExit`
        //   cleanup clears `RecklessDashDoubledBolts`, a buffered duplicate
        //   still sitting in `Messages<BoltLost>` can be surfaced by the
        //   reader cursor on the next `Playing` frame. Without this local,
        //   that read would re-pass the set check and enqueue yet another
        //   duplicate. Local persistence is bounded by bolt spawn count
        //   over a full run (small); Bevy never reuses `Entity` generations.
        //
        // - `RecklessDashDoubledBolts` (shared resource, per-node): the
        //   observable state that tests assert on. Exists so tests can
        //   inspect doubled-bolts state and so future systems (FX, stat
        //   tracking) can query "has this bolt been doubled this node?"
        //   without depending on an internal Local.
        if !already_doubled_ever.insert(msg.bolt) {
            continue;
        }
        doubled.0.insert(msg.bolt);
        let duplicate = BoltLost {
            bolt:    msg.bolt,
            breaker: msg.breaker,
        };
        // Defer the write via `Commands` so the `MessageReader<BoltLost>`
        // (`Res<Messages<BoltLost>>`) borrow does not conflict with
        // `ResMut<Messages<BoltLost>>` in the same system.
        commands.queue(move |world: &mut World| {
            world.resource_mut::<Messages<BoltLost>>().write(duplicate);
        });
    }
}

/// Runs on `OnExit(NodeState::Playing)`. Clears the
/// `RecklessDashDoubledBolts` tracking set so per-node state does not leak
/// across nodes / runs. Runs unconditionally (no `run_if`) — safe no-op when
/// the set is already empty.
pub(crate) fn reckless_dash_cleanup_node(mut doubled: ResMut<RecklessDashDoubledBolts>) {
    doubled.0.clear();
}
