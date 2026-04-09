use bevy::prelude::*;

use super::{
    super::*,
    helpers::{Counter, increment},
};
use crate::state::types::{AppState, ChipSelectState, GameState, NodeState, RunState};

// ════════════════════════════════════════════════════════════════════
// Section C: State Navigation — in_state_node_playing()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 6: in_state_node_playing() drives into NodeState::Playing ──

#[test]
fn in_state_node_playing_sets_app_state_to_game() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Game,
        "in_state_node_playing() must set AppState::Game"
    );
}

#[test]
fn in_state_node_playing_sets_game_state_to_run() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Run,
        "in_state_node_playing() must set GameState::Run"
    );
}

#[test]
fn in_state_node_playing_sets_run_state_to_node() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    assert_eq!(
        *app.world().resource::<State<RunState>>().get(),
        RunState::Node,
        "in_state_node_playing() must set RunState::Node"
    );
}

#[test]
fn in_state_node_playing_sets_node_state_to_playing() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    assert_eq!(
        *app.world().resource::<State<NodeState>>().get(),
        NodeState::Playing,
        "in_state_node_playing() must set NodeState::Playing"
    );
}

#[test]
fn in_state_node_playing_chip_select_state_not_present() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    assert!(
        app.world()
            .get_resource::<State<ChipSelectState>>()
            .is_none(),
        "ChipSelectState should not exist when RunState is Node"
    );
}

// ── Behavior 7: state-gated system executes after in_state_node_playing ──

#[test]
fn state_gated_system_runs_in_node_playing() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment.run_if(in_state(NodeState::Playing)))
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "System gated on NodeState::Playing should execute after in_state_node_playing()"
    );
}

#[test]
fn system_gated_on_wrong_state_does_not_run() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment.run_if(in_state(NodeState::Loading)))
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Counter>().0,
        0,
        "System gated on NodeState::Loading should NOT run when in NodeState::Playing"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section D: State Navigation — in_state_chip_selecting()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 8: in_state_chip_selecting() drives into ChipSelectState::Selecting ──

#[test]
fn in_state_chip_selecting_sets_app_state_to_game() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .build();
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Game,
        "in_state_chip_selecting() must set AppState::Game"
    );
}

#[test]
fn in_state_chip_selecting_sets_game_state_to_run() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .build();
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Run,
        "in_state_chip_selecting() must set GameState::Run"
    );
}

#[test]
fn in_state_chip_selecting_sets_run_state_to_chip_select() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .build();
    assert_eq!(
        *app.world().resource::<State<RunState>>().get(),
        RunState::ChipSelect,
        "in_state_chip_selecting() must set RunState::ChipSelect"
    );
}

#[test]
fn in_state_chip_selecting_sets_chip_select_state_to_selecting() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .build();
    assert_eq!(
        *app.world().resource::<State<ChipSelectState>>().get(),
        ChipSelectState::Selecting,
        "in_state_chip_selecting() must set ChipSelectState::Selecting"
    );
}

#[test]
fn in_state_chip_selecting_node_state_not_present() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .build();
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "NodeState should not exist when RunState is ChipSelect"
    );
}

// ── Behavior 9: state-gated system executes after in_state_chip_selecting ──

#[test]
fn state_gated_system_runs_in_chip_selecting() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<Counter>()
        .with_system(
            Update,
            increment.run_if(in_state(ChipSelectState::Selecting)),
        )
        .build();
    app.update();
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "System gated on ChipSelectState::Selecting should execute"
    );
}

#[test]
fn system_gated_on_chip_select_loading_does_not_run() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<Counter>()
        .with_system(Update, increment.run_if(in_state(ChipSelectState::Loading)))
        .build();
    app.update();
    assert_eq!(
        app.world().resource::<Counter>().0,
        0,
        "System gated on ChipSelectState::Loading should NOT run when in Selecting"
    );
}

// ── Behavior 9b: chaining navigations — last one wins ──

#[test]
fn chained_navigation_last_wins() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .in_state_chip_selecting()
        .build();
    assert_eq!(
        *app.world().resource::<State<RunState>>().get(),
        RunState::ChipSelect,
        "Last navigation (chip_selecting) should win over earlier (node_playing)"
    );
    assert_eq!(
        *app.world().resource::<State<ChipSelectState>>().get(),
        ChipSelectState::Selecting,
    );
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "NodeState should not exist after navigating away from RunState::Node"
    );
}
