//! Scenario lifecycle plugin and system registration.

use bevy::prelude::*;
use breaker::{
    bolt::BoltSystems,
    breaker::BreakerSystems,
    run::node::{messages::SpawnNodeComplete, sets::NodeSystems},
    shared::GameState,
    ui::messages::ChipSelected,
};

use super::{
    debug_setup::{apply_debug_setup, deferred_debug_setup, enforce_frozen_positions},
    entity_tagging::tag_game_entities,
    frame_control::{
        check_frame_limit, entered_playing, exit_on_run_end,
        mark_entered_playing_on_spawn_complete, restart_run_on_end, tick_scenario_frame,
    },
    frame_mutations::apply_debug_frame_mutations,
    input::{init_scenario_input, inject_scenario_input},
    menu_bypass::{auto_skip_chip_select, bypass_menu_to_playing, seed_initial_chips},
    pending_effects::{
        apply_pending_bolt_effects, apply_pending_breaker_effects, apply_pending_cell_effects,
        apply_pending_wall_effects,
    },
    perfect_tracking::{apply_perfect_tracking, update_force_bump_grade},
    types::{ChipSelectionIndex, ScenarioConfig},
};
use crate::invariants::{
    EntityLeakBaseline, PreviousGameState, ScenarioFrame, ScenarioStats, ViolationLog,
    check_aabb_matches_entity_dimensions, check_bolt_count_reasonable, check_bolt_in_bounds,
    check_bolt_speed_accurate, check_breaker_in_bounds, check_breaker_position_clamped,
    check_chain_arc_count_reasonable, check_chip_offer_expected, check_chip_stacks_consistent,
    check_gravity_well_count_reasonable, check_maxed_chip_never_offered, check_no_entity_leaks,
    check_no_nan, check_offering_no_duplicates, check_physics_frozen_during_pause,
    check_pulse_ring_accumulation, check_run_stats_monotonic, check_second_wind_wall_at_most_one,
    check_shield_charges_consistent, check_timer_monotonically_decreasing,
    check_timer_non_negative, check_valid_breaker_state, check_valid_state_transitions,
};

/// Plugin that drives the scenario lifecycle.
pub struct ScenarioLifecycle;

impl Plugin for ScenarioLifecycle {
    fn build(&self, app: &mut App) {
        let allow_early_end = app
            .world()
            .resource::<ScenarioConfig>()
            .definition
            .allow_early_end;

        register_scenario_resources(app);
        register_scenario_systems(app);

        if allow_early_end {
            app.add_systems(Update, exit_on_run_end.run_if(in_state(GameState::RunEnd)));
        } else {
            app.add_systems(OnEnter(GameState::RunEnd), restart_run_on_end);
        }
    }
}

/// Initialises all resources and messages required by the scenario lifecycle.
fn register_scenario_resources(app: &mut App) {
    app.init_resource::<ScenarioFrame>()
        .init_resource::<ViolationLog>()
        .init_resource::<PreviousGameState>()
        .init_resource::<EntityLeakBaseline>()
        .init_resource::<ScenarioStats>()
        .init_resource::<ChipSelectionIndex>()
        // Registered here (not just in game plugins) so isolated test apps work.
        .add_message::<SpawnNodeComplete>()
        .add_message::<ChipSelected>()
        // Needed by check_timer_monotonically_decreasing exemption logic.
        .add_message::<breaker::run::node::messages::ReverseTimePenalty>();
}

/// Registers all scenario systems: input, lifecycle hooks, invariant checkers.
fn register_scenario_systems(app: &mut App) {
    let chip_select_condition = in_state(GameState::ChipSelect)
        .and(resource_exists::<breaker::screen::chip_select::ChipOffers>);
    let playing_gate = |stats: Option<Res<ScenarioStats>>| stats.is_some_and(|s| s.entered_playing);
    // Invariant checkers run in two chained batches after setup. All checkers share
    // `ResMut<ViolationLog>`, so Bevy serialises them automatically within each batch.
    let checkers_a = (
        check_bolt_in_bounds,
        check_bolt_speed_accurate,
        check_bolt_count_reasonable,
        check_breaker_in_bounds,
        check_no_nan,
        check_timer_non_negative,
        check_valid_state_transitions,
        check_valid_breaker_state,
        check_timer_monotonically_decreasing,
        check_breaker_position_clamped,
        check_physics_frozen_during_pause,
    )
        .chain();
    let checkers_b = (
        check_no_entity_leaks,
        check_offering_no_duplicates,
        check_maxed_chip_never_offered,
        check_chip_stacks_consistent,
        check_run_stats_monotonic,
        check_second_wind_wall_at_most_one,
        check_shield_charges_consistent,
        check_pulse_ring_accumulation,
        check_chain_arc_count_reasonable,
    )
        .chain();
    let checkers_c = (
        check_aabb_matches_entity_dimensions,
        check_gravity_well_count_reasonable,
    )
        .chain();
    app.add_systems(OnEnter(GameState::MainMenu), bypass_menu_to_playing)
        .add_systems(
            Update,
            check_chip_offer_expected.run_if(chip_select_condition.clone()),
        )
        .add_systems(
            PostUpdate,
            auto_skip_chip_select.run_if(chip_select_condition),
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (
                seed_initial_chips,
                init_scenario_input,
                ApplyDeferred,
                tag_game_entities,
                ApplyDeferred,
                apply_debug_setup,
            )
                .chain()
                .after(BoltSystems::Reset)
                .after(BreakerSystems::Reset)
                .after(NodeSystems::InitTimer),
        )
        .add_systems(
            FixedPreUpdate,
            (
                inject_scenario_input,
                apply_perfect_tracking,
                update_force_bump_grade,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                (tick_scenario_frame, check_frame_limit)
                    .chain()
                    .run_if(entered_playing)
                    .before(BreakerSystems::Move),
                (
                    enforce_frozen_positions,
                    apply_debug_frame_mutations,
                    checkers_a,
                    checkers_b,
                    checkers_c,
                )
                    .chain()
                    .run_if(playing_gate)
                    .after(deferred_debug_setup)
                    .after(tag_game_entities)
                    .after(BreakerSystems::UpdateState)
                    .before(BoltSystems::BoltLost),
                tag_game_entities,
                deferred_debug_setup.after(tag_game_entities),
                apply_pending_bolt_effects.after(tag_game_entities),
                apply_pending_breaker_effects.after(tag_game_entities),
                apply_pending_cell_effects.after(tag_game_entities),
                apply_pending_wall_effects.after(tag_game_entities),
                mark_entered_playing_on_spawn_complete,
            ),
        );
}
