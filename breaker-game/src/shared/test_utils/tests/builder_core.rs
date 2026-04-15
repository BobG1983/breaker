use std::time::Duration;

use bevy::prelude::*;

use super::super::*;
use crate::state::types::{AppState, ChipSelectState, GameState, NodeState, RunEndState, RunState};

// ════════════════════════════════════════════════════════════════════
// Section A: TestAppBuilder Core Construction
// ════════════════════════════════════════════════════════════════════

// ── Behavior 1: new() returns a builder that produces a minimal app ──

#[test]
fn builder_new_produces_app_with_time_fixed_resource() {
    let app = TestAppBuilder::new().build();
    // MinimalPlugins provides Time<Fixed>; stub doesn't add MinimalPlugins,
    // so this should fail if the stub is a bare App::new().
    let time_fixed = app.world().get_resource::<Time<Fixed>>();
    assert!(
        time_fixed.is_some(),
        "App from TestAppBuilder::new().build() must have Time<Fixed> (MinimalPlugins)"
    );
}

#[test]
fn builder_new_time_fixed_has_default_timestep() {
    let app = TestAppBuilder::new().build();
    let time_fixed = app.world().resource::<Time<Fixed>>();
    let expected = Duration::from_secs_f64(1.0 / 64.0);
    assert_eq!(
        time_fixed.timestep(),
        expected,
        "Time<Fixed> timestep should be the Bevy default (1/64s), got {:?}",
        time_fixed.timestep()
    );
}

// ── Behavior 2: new() does not register states ──

#[test]
fn builder_new_does_not_register_app_state() {
    let app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<State<AppState>>().is_none(),
        "TestAppBuilder::new() should not register AppState"
    );
}

#[test]
fn builder_new_does_not_register_sub_states() {
    let app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<State<GameState>>().is_none(),
        "TestAppBuilder::new() should not register GameState"
    );
    assert!(
        app.world().get_resource::<State<RunState>>().is_none(),
        "TestAppBuilder::new() should not register RunState"
    );
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "TestAppBuilder::new() should not register NodeState"
    );
}

// ── Behavior 3: new() does not register messages ──

#[test]
fn builder_new_does_not_register_message_collector() {
    use crate::{cells::components::Cell, shared::death_pipeline::damage_dealt::DamageDealt};

    let app = TestAppBuilder::new().build();
    assert!(
        app.world()
            .get_resource::<MessageCollector<DamageDealt<Cell>>>()
            .is_none(),
        "TestAppBuilder::new() should not register any MessageCollector"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section B: State Hierarchy Registration
// ════════════════════════════════════════════════════════════════════

// ── Behavior 4: with_state_hierarchy() registers states ──

#[test]
fn with_state_hierarchy_registers_app_state() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.update();
    let state = app.world().get_resource::<State<AppState>>();
    assert!(
        state.is_some(),
        "with_state_hierarchy() must register AppState"
    );
    assert_eq!(
        *state.unwrap().get(),
        AppState::Loading,
        "AppState should default to Loading"
    );
}

#[test]
fn with_state_hierarchy_sub_states_not_present_in_default_parent() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.update();
    // GameState is a sub-state of AppState::Game, not AppState::Loading
    assert!(
        app.world().get_resource::<State<GameState>>().is_none(),
        "GameState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<RunState>>().is_none(),
        "RunState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "NodeState should not be present when AppState is Loading"
    );
    assert!(
        app.world()
            .get_resource::<State<ChipSelectState>>()
            .is_none(),
        "ChipSelectState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<RunEndState>>().is_none(),
        "RunEndState should not be present when AppState is Loading"
    );
}

// ── Behavior 5: with_state_hierarchy() typestate transition ──

#[test]
fn with_state_hierarchy_enables_state_navigation_methods() {
    // This test verifies the typestate transition at compile time.
    // If it compiles, the test passes (in_state_node_playing is only on WithStates).
    let _app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
}
