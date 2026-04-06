use bevy::prelude::*;

use super::system::*;
use crate::state::{
    run::resources::{NodeOutcome, NodeResult},
    types::RunState,
};

#[test]
fn plugin_builds() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin::default(),
        ))
        .add_plugins(StatePlugin)
        .update();
}

// ── Behavior 7a: Quit routes to RunState::Teardown ──────────────────

#[test]
fn resolve_node_next_state_quit_returns_teardown() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Quit,
        node_index: 0,
        transition_queued: false,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(
        next,
        RunState::Teardown,
        "NodeResult::Quit should route to RunState::Teardown"
    );
}

#[test]
fn resolve_node_next_state_quit_ignores_node_index_and_transition_queued() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Quit,
        node_index: 99,
        transition_queued: true,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(
        next,
        RunState::Teardown,
        "NodeResult::Quit should route to Teardown regardless of node_index or transition_queued"
    );
}

// ── Behavior 7b: InProgress routes to ChipSelect ────────────────────

#[test]
fn resolve_node_next_state_in_progress_returns_chip_select() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::InProgress,
        node_index: 0,
        transition_queued: false,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::ChipSelect);
}

// ── Behavior 7c: Won routes to RunEnd ───────────────────────────────

#[test]
fn resolve_node_next_state_won_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Won,
        node_index: 8,
        transition_queued: false,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}

// ── Behavior 7d: TimerExpired routes to RunEnd ──────────────────────

#[test]
fn resolve_node_next_state_timer_expired_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::TimerExpired,
        node_index: 3,
        transition_queued: false,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}

// ── Behavior 7e: LivesDepleted routes to RunEnd ─────────────────────

#[test]
fn resolve_node_next_state_lives_depleted_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::LivesDepleted,
        node_index: 1,
        transition_queued: false,
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}
