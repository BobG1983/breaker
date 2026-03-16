//! Node subdomain plugin registration.

use bevy::prelude::*;

use crate::{
    run::node::{
        messages::{NodeCleared, TimerExpired},
        resources::{ClearRemainingCount, NodeTimer},
        sets::NodeSystems,
        systems::{
            init_clear_remaining, init_node_timer, set_active_layout, spawn_cells_from_layout,
            tick_node_timer, track_node_completion,
        },
    },
    shared::{GameState, PlayingState},
};

/// Plugin for the node subdomain.
///
/// Owns node layout, timer, cell spawning, and completion tracking.
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClearRemainingCount>()
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
                    track_node_completion.in_set(NodeSystems::TrackCompletion),
                    tick_node_timer.in_set(NodeSystems::TickTimer),
                )
                    .run_if(in_state(PlayingState::Active)),
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
            .add_message::<crate::cells::messages::CellDestroyed>()
            .add_plugins(NodePlugin)
            .update();
    }
}
