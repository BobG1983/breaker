//! Tests for the `register()` / reset system: `EntropyEngineState` is cleared
//! on `OnEnter(NodeState::Playing)`.

use bevy::{prelude::*, state::app::StatesPlugin};

use crate::{
    effect::effects::entropy_engine::effect::*,
    state::types::{AppState, GameState, NodeState, RunState},
};

// -- Behavior 21: register() wires reset system for OnEnter(NodeState::Playing) --

fn test_app_with_reset() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.init_state::<AppState>();
    app.add_sub_state::<GameState>();
    app.add_sub_state::<RunState>();
    app.add_sub_state::<NodeState>();
    register(&mut app);
    app
}

fn enter_playing(app: &mut App) {
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

#[test]
fn reset_system_clears_cells_destroyed_on_node_start() {
    let mut app = test_app_with_reset();
    enter_playing(&mut app);

    let entity = app
        .world_mut()
        .spawn(EntropyEngineState { cells_destroyed: 7 })
        .id();

    // Transition out and back in to trigger OnEnter(PlayingState::Active) again
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let state = app.world().get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 0,
        "cells_destroyed should be reset to 0 on node start"
    );
}

#[test]
fn reset_system_clears_multiple_entities() {
    let mut app = test_app_with_reset();
    enter_playing(&mut app);

    let entity1 = app
        .world_mut()
        .spawn(EntropyEngineState { cells_destroyed: 7 })
        .id();
    let entity2 = app
        .world_mut()
        .spawn(EntropyEngineState {
            cells_destroyed: 15,
        })
        .id();

    // Transition out and back to trigger reset
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let state1 = app.world().get::<EntropyEngineState>(entity1).unwrap();
    let state2 = app.world().get::<EntropyEngineState>(entity2).unwrap();
    assert_eq!(
        state1.cells_destroyed, 0,
        "entity1 cells_destroyed should be reset"
    );
    assert_eq!(
        state2.cells_destroyed, 0,
        "entity2 cells_destroyed should be reset"
    );
}

// -- Behavior 22: reset system does not remove the component --

#[test]
fn reset_system_does_not_remove_component() {
    let mut app = test_app_with_reset();
    enter_playing(&mut app);

    let entity = app
        .world_mut()
        .spawn(EntropyEngineState {
            cells_destroyed: 15,
        })
        .id();

    // Leave NodeState::Playing, then re-enter to trigger OnEnter reset
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    assert!(
        app.world().get::<EntropyEngineState>(entity).is_some(),
        "EntropyEngineState component should still exist after reset (not removed)"
    );
    let state = app.world().get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 0,
        "cells_destroyed should be 0 after reset"
    );
}
