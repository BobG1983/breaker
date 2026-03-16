//! Run plugin registration.

use bevy::prelude::*;

use crate::{
    run::{
        messages::RunLost,
        node::{NodePlugin, NodeSystems},
        resources::RunState,
        systems::{
            advance_node, handle_node_cleared, handle_run_lost, handle_timer_expired,
            reset_run_state,
        },
    },
    shared::{GameRng, GameState, PlayingState},
};

/// Plugin for the run domain.
///
/// Owns run state, node sequencing, and delegates node internals to [`NodePlugin`].
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .init_resource::<GameRng>()
            .add_plugins(NodePlugin)
            .add_message::<RunLost>()
            .add_systems(
                FixedUpdate,
                (
                    handle_node_cleared.after(NodeSystems::TrackCompletion),
                    handle_timer_expired
                        .after(NodeSystems::TickTimer)
                        .after(handle_node_cleared),
                    handle_run_lost
                        .after(handle_node_cleared)
                        .after(handle_timer_expired),
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
