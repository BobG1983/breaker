//! Shared test fixtures for Drift hazard tests.
//!
//! State-hierarchy apps (`NodeState::Playing` and a non-Playing variant),
//! bolt spawn helpers, a variable-dt ticker, a seeded-RNG installer, and
//! `ActiveHazards` / `DriftConfig` / `DriftWind` installers. Drift emits no
//! messages, so unlike Overcharge or Cascade there is no message helper here.

use std::time::Duration;

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::system::{DriftConfig, DriftWind, drift_apply_force, drift_update_wind};
use crate::{
    bolt::components::Bolt,
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

/// Default builder: state hierarchy driven into `NodeState::Playing`,
/// `ActiveHazards`.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .build()
}

/// State-hierarchy app that is NOT driven into `NodeState::Playing`.
/// Used to test the `in_state(NodeState::Playing)` gate in Group D.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .build()
}

/// Spawns `(Bolt, Velocity2D(velocity))` and returns its id.
pub(super) fn spawn_bolt(app: &mut App, velocity: Vec2) -> Entity {
    app.world_mut().spawn((Bolt, Velocity2D(velocity))).id()
}

/// Seeds `GameRng` with a `ChaCha8Rng::seed_from_u64(seed)`.
pub(super) fn insert_rng(app: &mut App, seed: u64) {
    app.world_mut()
        .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
}

/// Sets the `Time<Fixed>` timestep to `dt`, accumulates one overstep of
/// `dt`, and runs one `app.update()`.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Canonical tuning used across most tests:
/// `{ force: 100.0, period_secs: 8.0, per_level_force: 33.3 }`.
pub(super) const fn canonical_config() -> DriftConfig {
    DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    }
}

/// Inserts the given `DriftConfig` as a resource.
pub(super) fn install_drift_config(app: &mut App, cfg: DriftConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts the given `DriftWind` as a resource.
pub(super) fn install_drift_wind(app: &mut App, wind: DriftWind) {
    app.world_mut().insert_resource(wind);
}

/// Adds `count` Drift stacks to `ActiveHazards`.
pub(super) fn add_drift_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Drift);
    }
}

/// Wires only `drift_update_wind` in `FixedUpdate` — bypasses the
/// `hazard_active` / `in_state` gates `register` installs. Used by Group B.
pub(super) fn wire_update_wind_only(app: &mut App) {
    app.add_systems(FixedUpdate, drift_update_wind);
}

/// Wires only `drift_apply_force` in `FixedUpdate` — bypasses the
/// `hazard_active` / `in_state` gates `register` installs. Used by Group C.
pub(super) fn wire_apply_force_only(app: &mut App) {
    app.add_systems(FixedUpdate, drift_apply_force);
}
