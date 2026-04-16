//! Shared test harness for magnetic field integration tests (Parts B-D).
//!
//! Provides `build_magnetic_test_app`, `advance_to_playing`,
//! `spawn_magnetic_cell`, `spawn_test_bolt`, and `tick_with_dt` helpers.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use crate::{
    cells::behaviors::magnetic::{
        components::{MagneticCell, MagneticField},
        systems::apply_magnetic_fields,
    },
    prelude::*,
};

/// Builds the integration test `App` with magnetic-specific wiring.
///
/// Registers `apply_magnetic_fields` in `FixedUpdate` with
/// `.run_if(in_state(NodeState::Playing))`. State hierarchy is initialized
/// but NOT navigated to Playing -- call `advance_to_playing()` after spawning.
pub(super) fn build_magnetic_test_app() -> App {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.add_systems(
        FixedUpdate,
        apply_magnetic_fields.run_if(in_state(NodeState::Playing)),
    );
    app
}

/// Walks `AppState::Game -> GameState::Run -> RunState::Node -> NodeState::Playing`.
pub(super) fn advance_to_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
}

/// Spawns a magnetic cell at `pos` with the given field parameters and
/// `Aabb2D` half-extents.
pub(super) fn spawn_magnetic_cell(
    app: &mut App,
    pos: Vec2,
    radius: f32,
    strength: f32,
    half_width: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            MagneticCell,
            MagneticField { radius, strength },
            Position2D(pos),
            Aabb2D::new(Vec2::ZERO, Vec2::splat(half_width)),
            Hp::new(20.0),
            KilledBy::default(),
        ))
        .id()
}

/// Spawns a test bolt at `pos` with the given velocity and base speed.
pub(super) fn spawn_test_bolt(app: &mut App, pos: Vec2, vel: Vec2, base_speed: f32) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            Position2D(pos),
            Velocity2D(vel),
            BaseSpeed(base_speed),
        ))
        .id()
}

/// Sets the fixed timestep to `dt`, accumulates one step, then runs update.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}
