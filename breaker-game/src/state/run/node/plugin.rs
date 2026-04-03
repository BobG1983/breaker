//! Node subdomain plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use crate::state::{
    run::{
        chip_select::messages::ChipSelected,
        node::{
            hud::{
                UiSystems,
                systems::{spawn_side_panels, spawn_timer_hud, update_timer_display},
            },
            messages::{
                ApplyTimePenalty, CellsSpawned, NodeCleared, ReverseTimePenalty, SpawnNodeComplete,
                TimerExpired,
            },
            resources::{ClearRemainingCount, NodeTimer, ScenarioLayoutOverride},
            sets::NodeSystems,
            systems::{
                apply_time_penalty, check_spawn_complete, init_clear_remaining, init_node_timer,
                reverse_time_penalty, set_active_layout, spawn_cells_from_layout, tick_node_timer,
                track_node_completion,
            },
        },
    },
    types::NodeState,
};

/// Plugin for the node subdomain.
///
/// Owns node layout, timer, cell spawning, completion tracking, and HUD.
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClearRemainingCount>()
            .init_resource::<NodeTimer>()
            .init_resource::<ScenarioLayoutOverride>()
            .add_message::<crate::cells::messages::CellDestroyedAt>()
            .add_message::<NodeCleared>()
            .add_message::<TimerExpired>()
            .add_message::<ApplyTimePenalty>()
            .add_message::<ReverseTimePenalty>()
            .add_message::<CellsSpawned>()
            .add_message::<SpawnNodeComplete>()
            .add_message::<ChipSelected>()
            .add_systems(
                OnEnter(NodeState::Loading),
                (
                    set_active_layout,
                    spawn_cells_from_layout.in_set(NodeSystems::Spawn),
                    init_clear_remaining,
                    init_node_timer.in_set(NodeSystems::InitTimer),
                )
                    .chain(),
            )
            // HUD — side panels + timer display
            .add_systems(
                OnEnter(NodeState::Loading),
                (
                    spawn_side_panels,
                    ApplyDeferred,
                    spawn_timer_hud.in_set(UiSystems::SpawnTimerHud),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                update_timer_display.run_if(in_state(NodeState::Playing)),
            )
            // Intentionally runs without NodeState::Playing guard — must catch spawn signals on the first frame of play.
            .add_systems(FixedUpdate, check_spawn_complete)
            .add_systems(
                FixedUpdate,
                (
                    track_node_completion.in_set(NodeSystems::TrackCompletion),
                    tick_node_timer.in_set(NodeSystems::TickTimer),
                    reverse_time_penalty
                        .in_set(NodeSystems::ApplyTimePenalty)
                        .after(NodeSystems::TickTimer)
                        .before(apply_time_penalty),
                    apply_time_penalty
                        .in_set(NodeSystems::ApplyTimePenalty)
                        .after(NodeSystems::TickTimer),
                )
                    .run_if(in_state(NodeState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{AppState, GamePhase, RunPhase};

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<NodeState>()
            .add_plugins(NodePlugin)
            .update();
    }
}
