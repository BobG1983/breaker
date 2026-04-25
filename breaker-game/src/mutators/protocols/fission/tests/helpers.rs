//! Shared test fixtures for Fission protocol tests.
//!
//! App builders (with/without `FissionConfig`, with/without `FissionCounter`,
//! and a `ChipSelecting` variant), canonical `FissionConfig`, counter /
//! active-protocols seeders, entity spawners that use the canonical
//! `Bolt::builder()` chain, `Destroyed<Cell>` writers, and assertion helpers.
//!
//! Canonical config is `kills_per_split: 8` (the design-doc worked-example
//! value). The RON asset's `kills_per_split: 10` is exercised only by
//! `ron_asset.rs` via `include_str!`.

use std::marker::PhantomData;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{FissionConfig, FissionCounter, activate, register};
use crate::{
    bolt::test_utils::default_bolt_definition,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Fission test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, `FissionCounter` initialised (to default
/// `kills: 0`), `Destroyed<Cell>` message registered, canonical
/// `FissionConfig { kills_per_split: 8 }` inserted, a `BoltRegistry` seeded
/// with the canonical `default_bolt_definition()`, and `register` called.
///
/// Does NOT seed `ActiveProtocols` with Fission — tests that need the
/// protocol active call [`seed_active_protocols_with_fission`].
pub(super) fn build_fission_app() -> App {
    let def = default_bolt_definition();
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .with_bolt_registry_entry(&def.name, def.clone())
        .build();
    app.world_mut().insert_resource(canonical_fission_config());
    register(&mut app);
    app
}

/// Same as [`build_fission_app`] but omits `FissionConfig`. Used to exercise
/// the harness-safe `Option<Res<FissionConfig>>` guard.
pub(super) fn build_fission_app_no_config() -> App {
    let def = default_bolt_definition();
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .with_bolt_registry_entry(&def.name, def.clone())
        .build();
    register(&mut app);
    app
}

/// Same as [`build_fission_app`] but omits `FissionCounter`. Used to exercise
/// the harness-safe `Option<ResMut<FissionCounter>>` guard.
pub(super) fn build_fission_app_no_counter() -> App {
    let def = default_bolt_definition();
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<Destroyed<Cell>>()
        .with_bolt_registry_entry(&def.name, def.clone())
        .build();
    app.world_mut().insert_resource(canonical_fission_config());
    register(&mut app);
    app
}

/// Same as [`build_fission_app`] but omits the `BoltRegistry` entirely. Used
/// to exercise the harness-safe `Option<Res<BoltRegistry>>` guard.
pub(super) fn build_fission_app_no_registry() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .build();
    app.world_mut().insert_resource(canonical_fission_config());
    register(&mut app);
    app
}

/// Same as [`build_fission_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used by the `in_state(NodeState::Playing)`
/// gate test.
pub(super) fn build_fission_app_in_chip_selecting() -> App {
    let def = default_bolt_definition();
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .with_bolt_registry_entry(&def.name, def.clone())
        .build();
    app.world_mut().insert_resource(canonical_fission_config());
    register(&mut app);
    app
}

// ── Config / counter helpers ────────────────────────────────────────────────

/// Canonical Fission config used across all system-behavior tests:
/// `FissionConfig { kills_per_split: 8 }`. Intentionally DIFFERS from the RON
/// asset's `kills_per_split: 10`, which is exercised only by `ron_asset.rs`.
pub(super) const fn canonical_fission_config() -> FissionConfig {
    FissionConfig { kills_per_split: 8 }
}

/// Inserts the given `FissionConfig` as a resource (overwrites existing).
pub(super) fn install_fission_config(app: &mut App, cfg: FissionConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Replaces the app's `FissionCounter` with `FissionCounter { kills }`.
pub(super) fn install_fission_counter(app: &mut App, kills: u32) {
    app.world_mut().insert_resource(FissionCounter { kills });
}

/// Inserts a Fission `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(Fission)` run-condition passes.
pub(super) fn seed_active_protocols_with_fission(app: &mut App, kills_per_split: u32) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Fission".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Fission { kills_per_split },
        });
}

/// Invokes `fission::activate` directly via a fresh `CommandQueue` so each
/// call has an independent, deterministic flush. Mirrors
/// `siphon::tests::helpers::activate_now`.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Entity spawners ─────────────────────────────────────────────────────────

/// Spawns a headless primary bolt at `pos` with velocity `vel` using
/// `Bolt::builder().at_position(pos).definition(&default_bolt_definition())
/// .with_velocity(Velocity2D(vel)).primary().headless().spawn(..)` plus
/// `world.flush()`. Returns the spawned `Entity`.
pub(super) fn spawn_bolt_at_with_velocity(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
    let def = default_bolt_definition();
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(pos)
        .definition(&def)
        .with_velocity(Velocity2D(vel))
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

/// Spawns a headless primary bolt with the given `BoundEffects` installed via
/// `.with_inherited_effects(&bound)`.
pub(super) fn spawn_bolt_with_bound_effects(
    app: &mut App,
    pos: Vec2,
    vel: Vec2,
    bound: BoundEffects,
) -> Entity {
    let def = default_bolt_definition();
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(pos)
        .definition(&def)
        .with_velocity(Velocity2D(vel))
        .primary()
        .with_inherited_effects(&bound)
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

/// Spawns a headless primary bolt and inserts `StagedEffects` directly via
/// `world.entity_mut(bolt).insert(staged.clone())`.
pub(super) fn spawn_bolt_with_staged_effects(
    app: &mut App,
    pos: Vec2,
    vel: Vec2,
    staged: StagedEffects,
) -> Entity {
    let entity = spawn_bolt_at_with_velocity(app, pos, vel);
    app.world_mut().entity_mut(entity).insert(staged);
    entity
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `Destroyed<Cell>` message with
/// `victim: Entity::PLACEHOLDER`, the given `killer`, `victim_pos: Vec2::ZERO`,
/// `killer_pos: None`, `_marker: PhantomData`.
pub(super) fn write_destroyed_cell(app: &mut App, killer: Option<Entity>) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim: Entity::PLACEHOLDER,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker: PhantomData,
        });
}

/// Writes `count` `Destroyed<Cell>` messages, each with the same `killer`.
pub(super) fn write_n_destroyed_cell(app: &mut App, count: u32, killer: Option<Entity>) {
    for _ in 0..count {
        write_destroyed_cell(app, killer);
    }
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Count of entities with the `Bolt` marker in the world.
pub(super) fn count_bolts(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count()
}

/// Returns the list of entities with `Bolt` in the world, filtered to exclude
/// the given `parent` entity. Convenient for finding the new (split) bolt.
pub(super) fn bolts_other_than(app: &mut App, parent: Entity) -> Vec<Entity> {
    let all: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .collect();
    all.into_iter().filter(|e| *e != parent).collect()
}

/// Returns `(Vec2, Vec2)` for the given bolt entity, unwrapping on absence.
pub(super) fn bolt_pos_and_vel(app: &App, bolt: Entity) -> (Vec2, Vec2) {
    let pos = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt should have Position2D")
        .0;
    let vel = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt should have Velocity2D")
        .0;
    (pos, vel)
}

/// Count of entities with `PrimaryBolt`.
pub(super) fn count_primary_bolts(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<crate::bolt::components::PrimaryBolt>>()
        .iter(app.world())
        .count()
}
