//! Run plugin registration.

use bevy::prelude::*;

use crate::{
    run::{
        messages::{NodeCleared, TimerExpired},
        node::{
            ClearRemainingCount, NodeSystems, NodeTimer,
            systems::{
                init_clear_remaining, init_node_timer, set_active_layout, spawn_cells_from_layout,
                tick_node_timer, track_node_completion,
            },
        },
        resources::RunState,
        systems::{advance_node, handle_node_cleared, handle_timer_expired, reset_run_state},
    },
    shared::{GameState, PlayingState},
};

/// Plugin for the run domain.
///
/// Owns run state, node timer, and node sequencing.
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .init_resource::<ClearRemainingCount>()
            .init_resource::<NodeTimer>()
            .add_message::<NodeCleared>()
            .add_message::<TimerExpired>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    set_active_layout,
                    spawn_cells_from_layout.in_set(NodeSystems::Spawn),
                    init_clear_remaining,
                    init_node_timer,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    track_node_completion,
                    handle_node_cleared,
                    tick_node_timer,
                    handle_timer_expired,
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            .add_systems(OnEnter(GameState::NodeTransition), advance_node)
            .add_systems(OnExit(GameState::MainMenu), reset_run_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // RunPlugin reads CellDestroyed messages from cells domain
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_plugins(RunPlugin)
            .update();
    }
}
