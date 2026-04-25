//! Shared test fixtures for Burnout protocol tests.

use std::time::Duration;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
    time::TimeUpdateStrategy,
};

use super::super::system::{
    BurnoutConfig, BurnoutDamageBoost, BurnoutHeat, BurnoutSpeedBoost, activate, register,
};
use crate::{
    bolt::components::BoltBaseDamage,
    breaker::messages::BumpGrade,
    effect_v3::{components::EffectSourceChip, effects::shockwave::components::ShockwaveSource},
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Burnout test app. Registers systems and installs canonical
/// [`BurnoutConfig`]. Does NOT seed [`ActiveProtocols`] — call
/// [`seed_active_protocols_with_burnout`] when the protocol-active run-if
/// must pass.
pub(super) fn build_burnout_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut().insert_resource(canonical_burnout_config());
    register(&mut app);
    app
}

/// Same as [`build_burnout_app`] but omits `BurnoutConfig`. Used to exercise
/// harness-safe `Option<Res<BurnoutConfig>>` early-return guard paths in
/// each reader system.
pub(super) fn build_burnout_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    register(&mut app);
    app
}

/// Same as [`build_burnout_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests.
pub(super) fn build_burnout_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut().insert_resource(canonical_burnout_config());
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Burnout config used across all system-behavior tests. Matches
/// the design-doc source of truth — `fill_duration: 4.0, drain_duration: 2.0,
/// still_threshold: 1.5, full_heat_damage_multiplier: 4.0,
/// speed_boost_duration: 2.0`.
pub(super) const fn canonical_burnout_config() -> BurnoutConfig {
    BurnoutConfig {
        fill_duration:               4.0,
        drain_duration:              2.0,
        still_threshold:             1.5,
        full_heat_damage_multiplier: 4.0,
        speed_boost_duration:        2.0,
    }
}

/// Inserts the given `BurnoutConfig` as a resource.
pub(super) fn install_burnout_config(app: &mut App, cfg: BurnoutConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts a Burnout `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(Burnout)` run-condition passes.
pub(super) fn seed_active_protocols_with_burnout(
    app: &mut App,
    fill_duration: f32,
    drain_duration: f32,
    still_threshold: f32,
    full_heat_damage_multiplier: f32,
    speed_boost_duration: f32,
) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Burnout".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Burnout {
                fill_duration,
                drain_duration,
                still_threshold,
                full_heat_damage_multiplier,
                speed_boost_duration,
            },
        });
}

/// Invokes `burnout::activate` directly via a fresh `CommandQueue` so each
/// call has an independent, deterministic flush. Mirrors the Reckless Dash
/// `activate_now` helper.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Entity spawners ─────────────────────────────────────────────────────────

/// Spawns `(Breaker, Position2D(pos), Velocity2D(Vec2::ZERO),
/// BurnoutHeat::default())`.
pub(super) fn spawn_breaker_stationary(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(pos),
            Velocity2D(Vec2::ZERO),
            BurnoutHeat::default(),
        ))
        .id()
}

/// Spawns `(Breaker, Position2D(pos), Velocity2D(velocity),
/// BurnoutHeat::default())`.
pub(super) fn spawn_breaker_moving(app: &mut App, pos: Vec2, velocity: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(pos),
            Velocity2D(velocity),
            BurnoutHeat::default(),
        ))
        .id()
}

/// Spawns `(Bolt, BoltBaseDamage(base))`.
pub(super) fn spawn_bolt_with_base_damage(app: &mut App, base: f32) -> Entity {
    app.world_mut().spawn((Bolt, BoltBaseDamage(base))).id()
}

/// Spawns a bare empty entity — tests reuse this as a "cell" stand-in.
pub(super) fn spawn_cell_empty(app: &mut App) -> Entity {
    app.world_mut().spawn_empty().id()
}

// ── Direct component mutation helpers ───────────────────────────────────────

/// Overwrites the `BurnoutHeat` component on the given breaker to the given
/// field values. Bypasses production systems so tests can set preconditions
/// explicitly.
pub(super) fn set_heat_state(
    app: &mut App,
    breaker: Entity,
    heat: f32,
    still_timer: f32,
    mega_bump_charged: bool,
) {
    app.world_mut().entity_mut(breaker).insert(BurnoutHeat {
        heat,
        still_timer,
        mega_bump_charged,
    });
}

/// Inserts `BurnoutDamageBoost { multiplier }` directly on the given bolt.
pub(super) fn install_burnout_damage_boost(app: &mut App, bolt: Entity, multiplier: f32) {
    app.world_mut()
        .entity_mut(bolt)
        .insert(BurnoutDamageBoost { multiplier });
}

/// Inserts `BurnoutSpeedBoost { remaining }` directly on the given breaker.
pub(super) fn install_burnout_speed_boost(app: &mut App, breaker: Entity, remaining: f32) {
    app.world_mut()
        .entity_mut(breaker)
        .insert(BurnoutSpeedBoost { remaining });
}

/// Overwrites the breaker's `Velocity2D`. Used to flip a stationary breaker
/// into a moving one mid-test.
pub(super) fn set_breaker_velocity(app: &mut App, breaker: Entity, velocity: Vec2) {
    app.world_mut()
        .entity_mut(breaker)
        .insert(Velocity2D(velocity));
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message. `breaker` is forwarded verbatim
/// because `burnout_on_bump` must look up the breaker's `BurnoutHeat` by
/// `msg.breaker`.
pub(super) fn write_bump_performed(
    app: &mut App,
    breaker: Entity,
    bolt: Option<Entity>,
    grade: BumpGrade,
) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker,
        });
}

/// Writes a single `BoltImpactCell` message with placeholder `impact_normal`
/// and zero `piercing_remaining` (Burnout's amplifier does not inspect
/// either).
pub(super) fn write_bolt_impact_cell(app: &mut App, cell: Entity, bolt: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltImpactCell>>()
        .write(BoltImpactCell {
            cell,
            bolt,
            impact_normal: Vec2::ZERO,
            piercing_remaining: 0,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns the breaker's current `BurnoutHeat` snapshot, or `None` if the
/// component is absent.
pub(super) fn read_heat(app: &App, breaker: Entity) -> Option<BurnoutHeat> {
    app.world().get::<BurnoutHeat>(breaker).copied()
}

/// Returns `Some(boost.remaining)` if the breaker carries `BurnoutSpeedBoost`,
/// else `None`.
pub(super) fn read_speed_boost_remaining(app: &App, breaker: Entity) -> Option<f32> {
    app.world()
        .get::<BurnoutSpeedBoost>(breaker)
        .map(|b| b.remaining)
}

/// Returns `Some(boost.multiplier)` if the bolt carries `BurnoutDamageBoost`,
/// else `None`.
pub(super) fn read_damage_boost_multiplier(app: &App, bolt: Entity) -> Option<f32> {
    app.world()
        .get::<BurnoutDamageBoost>(bolt)
        .map(|b| b.multiplier)
}

/// Returns every `DamageDealt<Cell>` message whose `source` matches the
/// builder-produced `protocol:burnout` source, collected from the
/// `MessageCollector` since the app was built. Used by amplification and
/// sentinel tests to assert that only Burnout-attributed damage is emitted
/// (and how much).
pub(super) fn collected_burnout_damage(app: &App) -> Vec<DamageDealt<Cell>> {
    let burnout_source = SourceId::protocol(ProtocolKind::Burnout).build();
    app.world()
        .resource::<MessageCollector<DamageDealt<Cell>>>()
        .0
        .iter()
        .filter(|msg| msg.source.as_ref() == Some(&burnout_source))
        .cloned()
        .collect()
}

/// Counts the number of entities in the world with a `ShockwaveSource`
/// marker component.
pub(super) fn count_shockwave_sources(app: &mut App) -> usize {
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>();
    q.iter(app.world()).count()
}

/// Returns every entity in the world carrying `ShockwaveSource`.
pub(super) fn shockwave_source_entities(app: &mut App) -> Vec<Entity> {
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<ShockwaveSource>>();
    q.iter(app.world()).collect()
}

/// Returns the `Position2D` of the given entity as a `Vec2`, or `None` if
/// the component is absent.
pub(super) fn shockwave_position(app: &App, entity: Entity) -> Option<Vec2> {
    app.world().get::<Position2D>(entity).map(|p| p.0)
}

/// Returns the `EffectSourceChip` inner string content on the given entity,
/// or `None` if the component is absent (or carries an empty inner).
pub(super) fn shockwave_source_chip(app: &App, entity: Entity) -> Option<String> {
    app.world()
        .get::<EffectSourceChip>(entity)
        .and_then(|c| c.0.as_ref().map(|s| s.0.clone().into_owned()))
}

// ── Time-based helpers ──────────────────────────────────────────────────────

/// Number of `FixedUpdate` ticks equivalent to `seconds` at the default 1/64
/// second timestep.
pub(super) fn ticks_for_seconds(seconds: f32) -> u32 {
    (seconds * 64.0).round() as u32
}

/// Runs `n` consecutive `FixedUpdate` ticks.
pub(super) fn tick_n(app: &mut App, n: u32) {
    for _ in 0..n {
        tick(app);
    }
}
