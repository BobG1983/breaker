//! Node subdomain plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};
use rantzsoft_stateflow::{Route, RoutingTableAppExt, cleanup_on_exit};

use crate::{
    prelude::*,
    state::run::node::{
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
            all_animate_in_complete, apply_time_penalty, check_spawn_complete,
            init_clear_remaining, init_node_timer, reverse_time_penalty, set_active_layout,
            spawn_cells_from_layout, tick_node_timer, track_node_completion,
        },
    },
};

/// Plugin for the node subdomain.
///
/// Owns node layout, timer, cell spawning, completion tracking, and HUD.
pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        // NodeState routes — node lifecycle
        // Loading → AnimateIn: message-triggered (check_spawn_complete sends ChangeState)
        app.add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn));
        // AnimateIn → Playing: message-triggered (all_animate_in_complete sends ChangeState)
        app.add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing));
        // Playing → AnimateOut: message-triggered (handle_node_cleared etc. send ChangeState)
        app.add_route(Route::from(NodeState::Playing).to(NodeState::AnimateOut));
        // AnimateOut → Teardown: pass-through
        app.add_route(
            Route::from(NodeState::AnimateOut)
                .to(NodeState::Teardown)
                .when(|_| true),
        );
        app.add_systems(
            OnEnter(NodeState::Teardown),
            cleanup_on_exit::<NodeState>.in_set(NodeSystems::Cleanup),
        );

        app.init_resource::<ClearRemainingCount>()
            .init_resource::<NodeTimer>()
            .init_resource::<ScenarioLayoutOverride>()
            .add_message::<CellDestroyedAt>()
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
                all_animate_in_complete.run_if(in_state(NodeState::AnimateIn)),
            )
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
    use crate::state::types::{AppState, GameState, RunState};

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_plugins(
                rantzsoft_stateflow::RantzStateflowPlugin::new().register_state::<NodeState>(),
            )
            .add_plugins(NodePlugin)
            .update();
    }
}
