//! Shared test fixtures for Gravity Surge hazard tests.
//!
//! State-hierarchy apps (`NodeState::Playing` and a non-Playing variant),
//! bolt + well spawn helpers, a `Destroyed<Cell>` writer, a variable-dt
//! ticker, canonical tuning, config / stack installers, and per-group
//! wiring helpers. Also provides an `activate_now` helper that routes
//! through a `Commands` queue so the `activate` function runs via the
//! normal path (mirrors Drift).

use std::{marker::PhantomData, time::Duration};

use bevy::{
    ecs::world::CommandQueue,
    prelude::{Messages, *},
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::system::{
    GravitySurgeConfig, GravityWell, activate, despawn_expired_gravity_wells, gravity_well_pull,
    spawn_gravity_wells,
};
use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Default builder: state hierarchy driven into `NodeState::Playing`,
/// `ActiveHazards`, `Destroyed<Cell>` message queue.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

/// State-hierarchy app that is NOT driven into `NodeState::Playing`. Used
/// to test the `in_state(NodeState::Playing)` gate in Group F.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build()
}

/// Spawns `(Bolt, Position2D(position), Velocity2D(velocity))`.
pub(super) fn spawn_bolt(app: &mut App, position: Vec2, velocity: Vec2) -> Entity {
    app.world_mut()
        .spawn((Bolt, Position2D(position), Velocity2D(velocity)))
        .id()
}

/// Spawns `(GravityWell { strength, remaining }, Position2D(position))`.
pub(super) fn spawn_well(app: &mut App, position: Vec2, strength: f32, remaining: f32) -> Entity {
    app.world_mut()
        .spawn((
            GravityWell {
                strength,
                remaining,
            },
            Position2D(position),
        ))
        .id()
}

/// Writes a `Destroyed<Cell>` at `pos` with a placeholder victim.
pub(super) fn write_cell_destroyed(app: &mut App, pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: pos,
            killer_pos: None,
            _marker:    PhantomData,
        });
}

/// Sets the `Time<Fixed>` timestep to `dt`, accumulates one overstep of
/// `dt`, and runs one `app.update()`.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    // WARNING: do NOT call with `Duration::from_nanos(1)` or other tiny
    // values. Bevy's fixed-timestep accumulator enters a hang / spin when
    // `dt` is smaller than the accumulator's resolution threshold; stick
    // to `Duration::from_secs_f32(...)` or `Duration::from_millis(...)`
    // for sub-frame tests.
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Canonical tuning used across most tests:
/// `{ base_duration_secs: 2.0, per_level_duration_secs: 1.0,
/// base_strength: 500.0, per_level_strength_frac: 0.5 }`.
pub(super) const fn canonical_config() -> GravitySurgeConfig {
    GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    }
}

/// Inserts the given `GravitySurgeConfig` as a resource.
pub(super) fn install_gravity_surge_config(app: &mut App, cfg: GravitySurgeConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Adds `count` `GravitySurge` stacks to `ActiveHazards`.
pub(super) fn add_gravity_surge_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::GravitySurge);
    }
}

/// Wires only `spawn_gravity_wells` in `FixedUpdate` — bypasses gates.
pub(super) fn wire_spawn_only(app: &mut App) {
    app.add_systems(FixedUpdate, spawn_gravity_wells);
}

/// Wires only `gravity_well_pull` in `FixedUpdate` — bypasses gates.
pub(super) fn wire_pull_only(app: &mut App) {
    app.add_systems(FixedUpdate, gravity_well_pull);
}

/// Wires only `despawn_expired_gravity_wells` in `FixedUpdate` — bypasses gates.
pub(super) fn wire_despawn_only(app: &mut App) {
    app.add_systems(FixedUpdate, despawn_expired_gravity_wells);
}

/// Invoke `activate` directly against the world without using the `Update`
/// schedule. Multiple `app.add_systems(Update, closure)` calls in a single
/// test all run on every `app.update()` with undefined ordering, so the
/// "last system" pattern for overwrites is unreliable. This helper builds
/// a standalone `CommandQueue`, invokes `activate`, and applies — giving
/// each call an independent, deterministic flush.
pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

/// Seeds `GameRng` with a `ChaCha8Rng::seed_from_u64(seed)`. Used by
/// Group H (synergy with Drift, whose `drift_update_wind` reads `GameRng`).
pub(super) fn insert_seeded_rng(app: &mut App, seed: u64) {
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
}
