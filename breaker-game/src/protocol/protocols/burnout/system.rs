//! Burnout protocol — heat-gauge rhythm mechanic.
//!
//! Design doc: `docs/design/protocols/burnout.md`.

use std::marker::PhantomData;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE, sets::BoltSystems},
    breaker::sets::BreakerSystems,
    effect_v3::{
        EffectV3Systems, commands::EffectCommandsExt, effects::shockwave::ShockwaveConfig,
        types::EffectType,
    },
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::{ActiveProtocols, protocol_active},
    },
};

// ── BurnoutConfig ───────────────────────────────────────────────────────────

/// Per-run Burnout tuning extracted from `ProtocolTuning::Burnout` at
/// activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BurnoutConfig {
    /// Seconds of continuous movement required to fill heat from 0.0 to 1.0.
    pub(crate) fill_duration:               f32,
    /// Seconds of continuous stillness required to drain heat from 1.0 to 0.0.
    pub(crate) drain_duration:              f32,
    /// Seconds of stillness before the instant-drain + speed-boost fires.
    pub(crate) still_threshold:             f32,
    /// Multiplier applied to `BoltBaseDamage` on a mega-bump cell impact.
    pub(crate) full_heat_damage_multiplier: f32,
    /// Seconds the `BurnoutSpeedBoost` component lasts after still-threshold.
    pub(crate) speed_boost_duration:        f32,
}

// ── BurnoutHeat ─────────────────────────────────────────────────────────────

/// Per-breaker heat gauge. Fills while the breaker moves, drains while still,
/// and tracks how long the breaker has been still. When heat reaches 1.0 the
/// breaker arms a mega-bump (`mega_bump_charged = true`) — the next
/// `BumpPerformed` consumes the charge.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct BurnoutHeat {
    /// Current heat — `[0.0, 1.0]`, clamped by the update system.
    pub heat:              f32,
    /// Seconds accumulated while stationary — reset to 0.0 on any moving
    /// tick. When this crosses `config.still_threshold`, the instant-drain +
    /// speed-boost fires.
    pub still_timer:       f32,
    /// `true` once `heat == 1.0`, consumed on the next bump.
    pub mega_bump_charged: bool,
}

// ── BurnoutSpeedBoost ───────────────────────────────────────────────────────

/// Per-breaker component installed by the still-threshold drain. Decrements
/// each tick; removed when `remaining` reaches 0.0.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct BurnoutSpeedBoost {
    /// Seconds remaining on this boost — counted down by
    /// `burnout_tick_speed_boost`.
    pub(crate) remaining: f32,
}

// ── BurnoutDamageBoost ──────────────────────────────────────────────────────

/// Per-bolt single-shot amplified-damage marker inserted by `burnout_on_bump`
/// on a mega-bump consume and consumed by `burnout_amplify_damage` on the
/// bolt's next cell impact.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct BurnoutDamageBoost {
    /// Multiplier carried by this boost (copied from
    /// `BurnoutConfig::full_heat_damage_multiplier` at insert time).
    pub multiplier: f32,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts `BurnoutConfig` from `ProtocolTuning::Burnout`. Warns and no-ops
/// on a non-Burnout tuning variant, leaving any existing `BurnoutConfig`
/// intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Burnout {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    } = *tuning
    else {
        warn!("burnout::activate called with non-Burnout tuning");
        return;
    };
    commands.insert_resource(BurnoutConfig {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Burnout's runtime systems with the correct schedules, run-ifs,
/// and ordering.
///
/// `FixedUpdate` non-reader systems — gated by
/// `protocol_active(ProtocolKind::Burnout)` AND `in_state(NodeState::Playing)`:
/// - `burnout_tick_speed_boost` — before `BreakerSystems::Move`.
/// - `burnout_update_heat` — after `BreakerSystems::Move`, before
///   `burnout_on_bump`.
///
/// `FixedUpdate` reader systems — intentionally ungated at the tuple level.
/// A `.run_if(...)` gate suppresses the system's *execution* but does NOT
/// advance its `MessageReader` cursor, so messages sent during gated-off
/// frames remain in Bevy's two-frame message buffer and get retroactively
/// consumed the tick the gate opens. Reader systems instead enforce the
/// `ActiveProtocols` / `NodeState::Playing` gate in-body via an immediate
/// `reader.clear()` + return when inactive, draining the buffer every tick:
/// - `burnout_on_bump` — after `BreakerSystems::GradeBump`, after
///   `burnout_update_heat`.
/// - `burnout_amplify_damage` — after `BoltSystems::CellCollision`.
///
/// The explicit `burnout_update_heat → burnout_on_bump` edge is required:
/// `BreakerSystems::Move` and `BreakerSystems::GradeBump` have no relative
/// ordering (both only `.after(update_bump)`), so without this edge the
/// scheduler may place `on_bump` before `update_heat`. In that ordering a
/// stationary-breaker still-timer tick after `on_bump` resets it would
/// advance `still_timer` past 0.0 in the same tick the charge was consumed,
/// breaking the "reset to default" contract. Running `update_heat` first
/// establishes the current-tick heat state; then `on_bump` decides whether
/// to consume `mega_bump_charged` and reset the gauge.
///
/// `OnExit(NodeState::Playing)` (no run-if):
/// - `burnout_cleanup_node` — removes `BurnoutHeat`, `BurnoutSpeedBoost`, and
///   `BurnoutDamageBoost` from every entity that carries them.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            burnout_tick_speed_boost.before(BreakerSystems::Move),
            burnout_update_heat
                .after(BreakerSystems::Move)
                .before(burnout_on_bump),
        )
            .run_if(protocol_active(ProtocolKind::Burnout))
            .run_if(in_state(NodeState::Playing)),
    )
    .add_systems(
        FixedUpdate,
        (
            burnout_on_bump.after(BreakerSystems::GradeBump),
            burnout_amplify_damage
                .after(BoltSystems::CellCollision)
                .before(EffectV3Systems::Bridge),
        ),
    )
    .add_systems(OnExit(NodeState::Playing), burnout_cleanup_node);
}

// ── System 1 — burnout_update_heat ──────────────────────────────────────────

/// Per-tick breaker heat-gauge update. Fills while moving, drains while still,
/// clamps to `[0.0, 1.0]`, arms `mega_bump_charged` on full, tracks the
/// still-timer, and on `still_threshold` crossing fires an instant drain +
/// installs `BurnoutSpeedBoost` + dispatches a shockwave.
///
/// Harness-safe: early-returns when `BurnoutConfig` is absent.
///
/// Lazy-insert: breakers without `BurnoutHeat` get `BurnoutHeat::default()`
/// inserted via `Commands` on their first encounter; the component becomes
/// visible on the next `FixedUpdate` tick.
pub(crate) fn burnout_update_heat(
    time: Res<Time<Fixed>>,
    config: Option<Res<BurnoutConfig>>,
    mut breakers: Query<(Entity, &Velocity2D, Option<&mut BurnoutHeat>), With<Breaker>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        return;
    };
    let dt = time.delta_secs();
    for (entity, velocity, heat_opt) in &mut breakers {
        let Some(mut heat) = heat_opt else {
            commands.entity(entity).insert(BurnoutHeat::default());
            continue;
        };
        let speed_sq = velocity.0.length_squared();
        if speed_sq > f32::EPSILON {
            // Moving branch — fill heat, reset still timer.
            if config.fill_duration > f32::EPSILON {
                heat.heat = (heat.heat + dt / config.fill_duration).min(1.0);
            }
            heat.still_timer = 0.0;
            if heat.heat >= 1.0 - f32::EPSILON {
                heat.mega_bump_charged = true;
            }
        } else {
            // Stationary branch — drain heat, advance still timer, check threshold.
            if config.drain_duration > f32::EPSILON {
                heat.heat = (heat.heat - dt / config.drain_duration).max(0.0);
            }
            heat.still_timer += dt;
            // Still-threshold fire requires heat > 0: draining an already-empty
            // gauge is not a "burnout" event — there is nothing to convert into
            // a speed boost.
            if heat.heat > 0.0 && heat.still_timer >= config.still_threshold {
                heat.heat = 0.0;
                heat.still_timer = 0.0;
                heat.mega_bump_charged = false;
                commands.entity(entity).insert(BurnoutSpeedBoost {
                    remaining: config.speed_boost_duration,
                });
            }
        }
    }
}

// ── System 2 — burnout_on_bump ──────────────────────────────────────────────

/// Consumes `BumpPerformed` messages. On a breaker with
/// `BurnoutHeat::mega_bump_charged == true`, installs `BurnoutDamageBoost` on
/// the bolt, resets the breaker's `BurnoutHeat` (heat 0.0, charge cleared),
/// and dispatches a shockwave at the breaker's position via
/// `commands.fire_effect`.
///
/// Skips (`continue`) when:
/// - `msg.bolt` is `None` (spectator bump).
/// - Breaker lookup fails (despawned or missing `BurnoutHeat`).
/// - `mega_bump_charged` is `false`.
///
/// Harness-safe: if `BurnoutConfig` is absent, clears the reader and returns
/// so buffered messages do not leak into a later frame that does have the
/// resource.
///
/// Gated in-body: this system now runs every `FixedUpdate` tick. When
/// Burnout is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `BumpPerformed` messages cannot leak retroactively when the protocol
/// activates on a later frame.
pub(crate) fn burnout_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<BurnoutConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    mut breakers: Query<&mut BurnoutHeat, With<Breaker>>,
    mut commands: Commands,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::Burnout))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok(mut heat) = breakers.get_mut(msg.breaker) else {
            continue;
        };
        if !heat.mega_bump_charged {
            continue;
        }
        // Install damage boost on the bolt (despawned bolt tolerated).
        if let Ok(mut entity) = commands.get_entity(bolt) {
            entity.insert(BurnoutDamageBoost {
                multiplier: config.full_heat_damage_multiplier,
            });
        }
        // Consume charge — reset heat state to default (heat + timer + charge).
        heat.heat = 0.0;
        heat.still_timer = 0.0;
        heat.mega_bump_charged = false;
        // Dispatch shockwave at breaker's position via fire_effect — the
        // shockwave's `Fireable::fire` impl snapshots Position2D itself.
        commands.fire_effect(
            msg.breaker,
            EffectType::Shockwave(ShockwaveConfig {
                base_range:      OrderedFloat(64.0),
                range_per_level: OrderedFloat(16.0),
                stacks:          1,
                speed:           OrderedFloat(200.0),
            }),
            SourceId::protocol(ProtocolKind::Burnout)
                .action("shockwave")
                .build()
                .0
                .into_owned(),
        );
    }
}

// ── System 3 — burnout_amplify_damage ───────────────────────────────────────

/// Consumes `BoltImpactCell` messages. On a bolt carrying
/// `BurnoutDamageBoost`, emits an amplified `DamageDealt<Cell>` with
/// `amount = base_damage * boost.multiplier` and
/// `source = Some(SourceId::protocol(Burnout).build())` — gated on
/// `amount > 0.0`. Removes the
/// `BurnoutDamageBoost` from the bolt unconditionally (single-shot, even when
/// emission is gated off by the `amount > 0.0` guard).
///
/// Pierce guard: `bolt_cell_collision` can emit multiple `BoltImpactCell`
/// messages for the same bolt in a single frame (pierce-through). The
/// `BurnoutDamageBoost` removal is deferred via `commands`, so subsequent
/// iterations in the same invocation would otherwise still see the boost.
/// `amplified_this_frame` tracks already-amplified bolts within THIS
/// invocation so only ONE amplified emit is produced per boost.
///
/// Harness-safe: if `BurnoutConfig` is absent, clears the reader and returns
/// so buffered messages do not leak into a later frame that does have the
/// resource. Boost is NOT consumed in this path.
///
/// Gated in-body: this system now runs every `FixedUpdate` tick. When
/// Burnout is not active or `NodeState` is not `Playing`, it drains the
/// `MessageReader` via `reader.clear()` and returns so buffered
/// `BoltImpactCell` messages cannot be retroactively amplified when the
/// protocol activates on a later frame.
pub(crate) fn burnout_amplify_damage(
    mut reader: MessageReader<BoltImpactCell>,
    config: Option<Res<BurnoutConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    bolts: Query<(&BurnoutDamageBoost, Option<&BoltBaseDamage>)>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::Burnout))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    if config.is_none() {
        reader.clear();
        return;
    }
    // `Vec` (not `HashSet`) because cardinality is almost always 0 or 1: a
    // single-bolt game with one `BoltImpactCell` message per frame. Linear
    // `contains` beats hash overhead at this scale.
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
                source: Some(SourceId::protocol(ProtocolKind::Burnout).build()),
                _marker: PhantomData,
            });
        }
        // Unconditionally consume the boost — single-shot regardless of the
        // `amount > 0.0` emission gate.
        if let Ok(mut entity) = commands.get_entity(msg.bolt) {
            entity.remove::<BurnoutDamageBoost>();
        }
        amplified_this_frame.push(msg.bolt);
    }
}

// ── System 4 — burnout_tick_speed_boost ─────────────────────────────────────

/// Decrements `BurnoutSpeedBoost.remaining` by `delta_secs` each tick, and
/// removes the component when `remaining` reaches `0.0`.
///
/// Harness-safe: early-returns when `BurnoutConfig` is absent.
pub(crate) fn burnout_tick_speed_boost(
    time: Res<Time<Fixed>>,
    config: Option<Res<BurnoutConfig>>,
    mut boosts: Query<(Entity, &mut BurnoutSpeedBoost)>,
    mut commands: Commands,
) {
    if config.is_none() {
        return;
    }
    let dt = time.delta_secs();
    for (entity, mut boost) in &mut boosts {
        boost.remaining -= dt;
        if boost.remaining <= 0.0
            && let Ok(mut e) = commands.get_entity(entity)
        {
            e.remove::<BurnoutSpeedBoost>();
        }
    }
}

// ── System 5 — burnout_cleanup_node ─────────────────────────────────────────

type BurnoutCleanupBreakerQuery<'w, 's> =
    Query<'w, 's, Entity, Or<(With<BurnoutHeat>, With<BurnoutSpeedBoost>)>>;

type BurnoutCleanupBoltQuery<'w, 's> = Query<'w, 's, Entity, With<BurnoutDamageBoost>>;

/// Runs on `OnExit(NodeState::Playing)`. Removes `BurnoutHeat`,
/// `BurnoutSpeedBoost`, and `BurnoutDamageBoost` from every entity that
/// carries them, guaranteeing per-node state does not leak across nodes /
/// runs. Runs unconditionally (no `run_if`).
pub(crate) fn burnout_cleanup_node(
    mut commands: Commands,
    breakers: BurnoutCleanupBreakerQuery,
    bolts: BurnoutCleanupBoltQuery,
) {
    for entity in &breakers {
        commands
            .entity(entity)
            .remove::<BurnoutHeat>()
            .remove::<BurnoutSpeedBoost>();
    }
    for entity in &bolts {
        commands.entity(entity).remove::<BurnoutDamageBoost>();
    }
}
