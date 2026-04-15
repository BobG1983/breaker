//! Run plugin registration.

use bevy::prelude::*;

use super::{
    chip_select::systems::{
        detect_first_evolution, snapshot_node_highlights, track_chips_collected,
    },
    definition::HighlightConfig,
    loading::systems::{capture_run_seed, generate_node_sequence_system, reset_run_state},
    messages::{HighlightTriggered, RunLost},
    node::{
        NodePlugin, NodeSystems,
        highlights::systems::{
            detect_close_save, detect_combo_king, detect_mass_destruction, detect_nail_biter,
            detect_pinball_wizard,
        },
        lifecycle::systems::{
            handle_breaker_death, handle_node_cleared, handle_run_lost, handle_timer_expired,
            reset_highlight_tracker, spawn_highlight_text,
        },
        tracking::systems::{
            track_bolts_lost, track_bumps, track_cells_destroyed, track_evolution_damage,
            track_node_cleared_stats, track_time_elapsed,
        },
    },
    resources::{DifficultyCurve, HighlightTracker, NodeOutcome},
    run_end::systems::detect_most_powerful_evolution,
    systems::{advance_node, hide_gameplay_entities, setup_run, show_gameplay_entities},
};
use crate::{
    prelude::*,
    shared::{RunSeed, death_pipeline::sets::DeathPipelineSystems},
};

/// Plugin for the run domain.
///
/// Owns run state, node sequencing, and delegates node internals to [`NodePlugin`].
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeOutcome>()
            .init_resource::<DifficultyCurve>()
            .init_resource::<GameRng>()
            .init_resource::<RunSeed>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightConfig>()
            .init_resource::<HighlightTracker>()
            .add_plugins(NodePlugin)
            .add_message::<RunLost>()
            .add_message::<HighlightTriggered>()
            // Breaker death handler — runs in the death pipeline's HandleKill
            // set, consuming `KillYourself<Breaker>` produced by
            // `detect_deaths::<Breaker>`. Emits `RunLost` for the run state
            // machine to turn into `NodeResult::LivesDepleted`.
            .add_systems(
                FixedUpdate,
                handle_breaker_death.in_set(DeathPipelineSystems::HandleKill),
            )
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
                    track_cells_destroyed.after(DeathPipelineSystems::HandleKill),
                    track_bumps,
                    track_bolts_lost,
                    track_time_elapsed,
                    track_evolution_damage.after(DeathPipelineSystems::ApplyDamage),
                    track_node_cleared_stats.after(NodeSystems::TrackCompletion),
                    // Highlight detection
                    detect_mass_destruction.after(DeathPipelineSystems::HandleKill),
                    detect_close_save.after(crate::breaker::BreakerSystems::GradeBump),
                    detect_combo_king.after(DeathPipelineSystems::HandleKill),
                    detect_pinball_wizard,
                    detect_nail_biter.after(NodeSystems::TrackCompletion),
                )
                    .run_if(in_state(NodeState::Playing)),
            )
            // In-game highlight juice (Update, NodeState::Playing)
            .add_systems(
                Update,
                spawn_highlight_text.run_if(in_state(NodeState::Playing)),
            )
            // Chip selection tracking + evolution detection (Update, ChipSelect state)
            .add_systems(
                Update,
                (
                    track_chips_collected,
                    detect_first_evolution,
                    snapshot_node_highlights,
                )
                    .run_if(in_state(ChipSelectState::Selecting)),
            )
            .add_systems(
                OnEnter(NodeState::Loading),
                (reset_highlight_tracker, capture_run_seed, setup_run),
            )
            .add_systems(OnEnter(RunEndState::Active), detect_most_powerful_evolution)
            .add_systems(OnExit(RunState::Node), hide_gameplay_entities)
            .add_systems(
                OnEnter(RunState::Node),
                (advance_node, show_gameplay_entities),
            )
            .add_systems(
                OnExit(MenuState::Main),
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
    use crate::{
        bolt::messages::BoltLost,
        chips::inventory::ChipInventory,
        shared::death_pipeline::{despawn_entity::DespawnEntity, kill_yourself::KillYourself},
    };

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<MenuState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_sub_state::<ChipSelectState>()
            .add_sub_state::<RunEndState>()
            .add_plugins(
                rantzsoft_stateflow::RantzStateflowPlugin::new()
                    .register_state::<NodeState>()
                    .register_state::<ChipSelectState>()
                    .register_state::<RunEndState>(),
            )
            // Messages read by run domain systems — registered explicitly here
            // instead of via DeathPipelinePlugin to keep the test harness minimal.
            .add_message::<DamageDealt<Cell>>()
            .add_message::<Destroyed<Cell>>()
            .add_message::<DespawnEntity>()
            .add_message::<BumpPerformed>()
            .add_message::<BoltLost>()
            .add_message::<BoltImpactBreaker>()
            .add_message::<BoltImpactCell>()
            .add_message::<ChipSelected>()
            .add_message::<KillYourself<Breaker>>()
            // Resources required by run domain systems
            .init_resource::<ChipInventory>()
            .init_resource::<PlayfieldConfig>()
            .add_plugins(RunPlugin)
            .update();
    }
}
