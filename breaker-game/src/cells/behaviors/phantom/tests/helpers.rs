//! Shared test harness for Phantom integration tests (Groups C-I).
//!
//! Provides `build_phantom_test_app`, `advance_to_playing`, `spawn_phantom_cell`,
//! and `tick_with_dt` helpers used across all phantom integration test groups.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::phantom::{
            components::{PhantomCell, PhantomConfig, PhantomPhase, PhantomTimer},
            systems::tick_phantom_phase,
        },
        test_utils::spawn_cell_in_world,
    },
    prelude::*,
};

/// Default cell dimensions for test spawns.
pub(super) const TEST_CELL_DIM: f32 = 10.0;

/// Spawns a phantom cell via `Cell::builder().phantom(starting_phase)` at the
/// given position with default phantom config (`cycle_secs=3.0`, `telegraph_secs=0.5`).
pub(super) fn spawn_phantom_cell(
    app: &mut App,
    pos: Vec2,
    starting_phase: PhantomPhase,
    hp: f32,
) -> Entity {
    spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .phantom(starting_phase)
            .position(pos)
            .dimensions(TEST_CELL_DIM, TEST_CELL_DIM)
            .hp(hp)
            .headless()
            .spawn(commands)
    })
}

/// Spawns a phantom cell by manually inserting components (no builder) for
/// tests that need precise timer/phase control.
pub(super) fn spawn_phantom_cell_raw(
    app: &mut App,
    phase: PhantomPhase,
    timer: f32,
    config: PhantomConfig,
    layers: CollisionLayers,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            PhantomCell,
            phase,
            PhantomTimer(timer),
            config,
            layers,
            Hp::new(20.0),
            KilledBy::default(),
        ))
        .id()
}

/// Builds the integration test `App` with phantom-specific wiring.
///
/// Registers `tick_phantom_phase` in `FixedUpdate` with
/// `.run_if(in_state(NodeState::Playing))`. State hierarchy is initialized
/// but NOT navigated to Playing — call `advance_to_playing()` after spawning.
pub(super) fn build_phantom_test_app() -> App {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.add_systems(
        FixedUpdate,
        tick_phantom_phase.run_if(in_state(NodeState::Playing)),
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

/// Sets the fixed timestep to `dt` and accumulates one step, then runs update.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}
