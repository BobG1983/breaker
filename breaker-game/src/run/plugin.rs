//! Run plugin registration.

use bevy::prelude::*;

use crate::{
    run::{
        messages::RunLost,
        node::{NodePlugin, NodeSystems},
        resources::{DifficultyCurve, HighlightTracker, RunState, RunStats},
        systems::{
            advance_node, capture_run_seed, generate_node_sequence_system, handle_node_cleared,
            handle_run_lost, handle_timer_expired, reset_highlight_tracker, reset_run_state,
            track_bolts_lost, track_bumps, track_cells_destroyed, track_chips_collected,
            track_node_cleared_stats, track_time_elapsed,
        },
    },
    shared::{GameRng, GameState, PlayingState, RunSeed},
};

/// Plugin for the run domain.
///
/// Owns run state, node sequencing, and delegates node internals to [`NodePlugin`].
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .init_resource::<DifficultyCurve>()
            .init_resource::<GameRng>()
            .init_resource::<RunSeed>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .add_plugins(NodePlugin)
            .add_message::<RunLost>()
            .add_systems(
                FixedUpdate,
                (
                    handle_node_cleared.after(NodeSystems::TrackCompletion),
                    handle_timer_expired
                        .after(NodeSystems::ApplyTimePenalty)
                        .after(handle_node_cleared),
                    handle_run_lost
                        .after(handle_node_cleared)
                        .after(handle_timer_expired),
                    // Stats accumulation (passive message readers)
                    track_cells_destroyed,
                    track_bumps,
                    track_bolts_lost,
                    track_time_elapsed,
                    track_node_cleared_stats.after(NodeSystems::TrackCompletion),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            // Chip selection tracking (Update schedule, ChipSelect state)
            .add_systems(
                Update,
                track_chips_collected.run_if(in_state(GameState::ChipSelect)),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (reset_highlight_tracker, capture_run_seed),
            )
            .add_systems(OnEnter(GameState::TransitionIn), advance_node)
            .add_systems(
                OnExit(GameState::MainMenu),
                (
                    reset_run_state,
                    generate_node_sequence_system.after(reset_run_state),
                ),
            );
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
            // Messages read by run domain systems
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_message::<crate::breaker::messages::BumpPerformed>()
            .add_message::<crate::physics::messages::BoltLost>()
            .add_message::<crate::ui::messages::ChipSelected>()
            // ChipInventory required by reset_run_state
            .init_resource::<crate::chips::inventory::ChipInventory>()
            .add_plugins(RunPlugin)
            .update();
    }
}
