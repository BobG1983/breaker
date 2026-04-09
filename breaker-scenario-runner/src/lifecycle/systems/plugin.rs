//! Scenario lifecycle plugin and system registration.

use std::collections::HashSet;

use bevy::prelude::*;
use breaker::{
    bolt::BoltSystems,
    breaker::BreakerSystems,
    state::{
        run::{
            chip_select::messages::ChipSelected,
            node::{messages::SpawnNodeComplete, sets::NodeSystems},
        },
        types::{ChipSelectState, GameState, MenuState, NodeState, RunEndState, RunState},
    },
};
use rantzsoft_stateflow::{routing_table::RoutingTable, transition::types::TransitionKind};

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
use crate::{
    invariants::{
        EntityLeakBaseline, ScenarioFrame, ScenarioStats, ViolationLog,
        check_aabb_matches_entity_dimensions, check_bolt_birthing_layers_zeroed,
        check_bolt_count_reasonable, check_bolt_in_bounds, check_bolt_speed_accurate,
        check_breaker_count_reasonable, check_breaker_in_bounds, check_breaker_position_clamped,
        check_chain_arc_count_reasonable, check_chip_offer_expected, check_chip_stacks_consistent,
        check_gravity_well_count_reasonable, check_maxed_chip_never_offered, check_no_entity_leaks,
        check_no_nan, check_offering_no_duplicates, check_pulse_ring_accumulation,
        check_run_stats_monotonic, check_second_wind_wall_at_most_one,
        check_shield_wall_at_most_one, check_timer_monotonically_decreasing,
        check_timer_non_negative, check_valid_breaker_state,
    },
    types::{InvariantKind, ScenarioDefinition},
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

        strip_all_transitions(app);
        register_scenario_resources(app);
        register_scenario_systems(app);

        if allow_early_end {
            app.add_systems(
                Update,
                exit_on_run_end.run_if(in_state(RunEndState::Active)),
            );
        } else {
            app.add_systems(OnEnter(RunEndState::Active), restart_run_on_end);
        }
    }
}

/// Strips all transition effects from all routing tables, replacing them with
/// instant state changes (`TransitionKind::None`).
///
/// Animated transitions pause `Time<Virtual>`, which blocks `FixedUpdate` and
/// prevents the scenario frame counter from advancing. Headless scenarios don't
/// need visual transitions — instant state changes are correct and fast.
fn strip_all_transitions(app: &mut App) {
    fn strip<S: States>(app: &mut App) {
        if let Some(mut table) = app.world_mut().get_resource_mut::<RoutingTable<S>>() {
            for route in table.routes.values_mut() {
                route.transition = TransitionKind::None;
            }
        }
    }
    strip::<GameState>(app);
    strip::<MenuState>(app);
    strip::<RunState>(app);
    strip::<NodeState>(app);
    strip::<ChipSelectState>(app);
    strip::<RunEndState>(app);
}

/// Initialises all resources and messages required by the scenario lifecycle.
fn register_scenario_resources(app: &mut App) {
    app.init_resource::<ScenarioFrame>()
        .init_resource::<ViolationLog>()
        .init_resource::<EntityLeakBaseline>()
        .init_resource::<ScenarioStats>()
        .init_resource::<ChipSelectionIndex>()
        // Registered here (not just in game plugins) so isolated test apps work.
        .add_message::<SpawnNodeComplete>()
        .add_message::<ChipSelected>()
        // Needed by check_timer_monotonically_decreasing exemption logic.
        .add_message::<breaker::state::run::node::messages::ReverseTimePenalty>();
}

/// Returns `true` when the scenario has entered the `Playing` phase,
/// gating invariant checkers until entities are fully initialised.
fn playing_gate(stats: Option<Res<ScenarioStats>>) -> bool {
    stats.is_some_and(|s| s.entered_playing)
}

/// Registers each `FixedUpdate` invariant checker that is in the active set.
///
/// Uses a macro to avoid repeating the identical ordering constraints
/// (`.run_if(playing_gate).after(...).before(...)`) for all 22 checkers.
fn register_active_checkers(app: &mut App, active: &HashSet<InvariantKind>) {
    macro_rules! register_checker {
        ($kind:expr, $system:expr) => {
            if active.contains(&$kind) {
                app.add_systems(
                    FixedUpdate,
                    $system
                        .run_if(playing_gate)
                        .after(apply_debug_frame_mutations)
                        .after(deferred_debug_setup)
                        .after(tag_game_entities)
                        .after(BreakerSystems::UpdateState)
                        .before(BoltSystems::BoltLost),
                );
            }
        };
    }

    register_checker!(InvariantKind::BoltInBounds, check_bolt_in_bounds);
    register_checker!(InvariantKind::BoltSpeedAccurate, check_bolt_speed_accurate);
    register_checker!(
        InvariantKind::BoltCountReasonable,
        check_bolt_count_reasonable
    );
    register_checker!(InvariantKind::BreakerInBounds, check_breaker_in_bounds);
    register_checker!(InvariantKind::NoEntityLeaks, check_no_entity_leaks);
    register_checker!(InvariantKind::NoNaN, check_no_nan);
    register_checker!(InvariantKind::TimerNonNegative, check_timer_non_negative);
    register_checker!(InvariantKind::ValidDashState, check_valid_breaker_state);
    register_checker!(
        InvariantKind::TimerMonotonicallyDecreasing,
        check_timer_monotonically_decreasing
    );
    register_checker!(
        InvariantKind::BreakerPositionClamped,
        check_breaker_position_clamped
    );
    register_checker!(
        InvariantKind::OfferingNoDuplicates,
        check_offering_no_duplicates
    );
    register_checker!(
        InvariantKind::MaxedChipNeverOffered,
        check_maxed_chip_never_offered
    );
    register_checker!(
        InvariantKind::ChipStacksConsistent,
        check_chip_stacks_consistent
    );
    register_checker!(InvariantKind::RunStatsMonotonic, check_run_stats_monotonic);
    register_checker!(
        InvariantKind::SecondWindWallAtMostOne,
        check_second_wind_wall_at_most_one
    );
    register_checker!(
        InvariantKind::ShieldWallAtMostOne,
        check_shield_wall_at_most_one
    );
    register_checker!(
        InvariantKind::PulseRingAccumulation,
        check_pulse_ring_accumulation
    );
    register_checker!(
        InvariantKind::ChainArcCountReasonable,
        check_chain_arc_count_reasonable
    );
    register_checker!(
        InvariantKind::AabbMatchesEntityDimensions,
        check_aabb_matches_entity_dimensions
    );
    register_checker!(
        InvariantKind::GravityWellCountReasonable,
        check_gravity_well_count_reasonable
    );
    register_checker!(
        InvariantKind::BreakerCountReasonable,
        check_breaker_count_reasonable
    );
    register_checker!(
        InvariantKind::BoltBirthingLayersZeroed,
        check_bolt_birthing_layers_zeroed
    );
}

/// Registers all scenario systems: input, lifecycle hooks, invariant checkers.
fn register_scenario_systems(app: &mut App) {
    let active = {
        let config = app.world().resource::<ScenarioConfig>();
        active_invariant_kinds(&config.definition)
    };

    let chip_select_condition = in_state(ChipSelectState::Selecting)
        .and(resource_exists::<breaker::state::run::chip_select::ChipOffers>);

    // ALWAYS register check_chip_offer_expected in Update
    app.add_systems(
        Update,
        check_chip_offer_expected.run_if(chip_select_condition.clone()),
    );

    // ALWAYS register auto_skip_chip_select in PostUpdate (not a checker)
    app.add_systems(
        PostUpdate,
        auto_skip_chip_select.run_if(chip_select_condition),
    );

    // ALWAYS register the frame mutation systems chained together
    app.add_systems(
        FixedUpdate,
        (enforce_frozen_positions, apply_debug_frame_mutations)
            .chain()
            .run_if(playing_gate)
            .after(deferred_debug_setup)
            .after(tag_game_entities)
            .after(BreakerSystems::UpdateState)
            .before(BoltSystems::BoltLost),
    );

    // Conditionally register active FixedUpdate checkers.
    // Bevy serializes them automatically due to ResMut<ViolationLog> conflict.
    register_active_checkers(app, &active);

    // Non-checker lifecycle systems (OnEnter, FixedPreUpdate, etc.)
    app.add_systems(OnEnter(MenuState::Main), bypass_menu_to_playing)
        .add_systems(
            OnEnter(NodeState::Loading),
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

/// Returns `true` for all `InvariantKind` variants that correspond to
/// FixedUpdate-batch checkers. Returns `false` only for `ChipOfferExpected`,
/// which runs on a different schedule (`Update` with a `run_if` condition).
///
/// Uses an exhaustive match so new variants produce a compile error.
pub(crate) const fn is_fixed_update_checker(kind: InvariantKind) -> bool {
    match kind {
        InvariantKind::BoltInBounds
        | InvariantKind::BoltSpeedAccurate
        | InvariantKind::BoltCountReasonable
        | InvariantKind::BreakerInBounds
        | InvariantKind::NoEntityLeaks
        | InvariantKind::NoNaN
        | InvariantKind::TimerNonNegative
        | InvariantKind::ValidDashState
        | InvariantKind::TimerMonotonicallyDecreasing
        | InvariantKind::BreakerPositionClamped
        | InvariantKind::OfferingNoDuplicates
        | InvariantKind::MaxedChipNeverOffered
        | InvariantKind::ChipStacksConsistent
        | InvariantKind::RunStatsMonotonic
        | InvariantKind::SecondWindWallAtMostOne
        | InvariantKind::ShieldWallAtMostOne
        | InvariantKind::PulseRingAccumulation
        | InvariantKind::ChainArcCountReasonable
        | InvariantKind::AabbMatchesEntityDimensions
        | InvariantKind::GravityWellCountReasonable
        | InvariantKind::BreakerCountReasonable
        | InvariantKind::BoltBirthingLayersZeroed => true,
        InvariantKind::ChipOfferExpected => false,
    }
}

/// Computes the set of `InvariantKind` variants that should have their
/// `FixedUpdate` checkers registered for this scenario.
///
/// Takes the union of `disallowed_failures` and `allowed_failures` (if
/// present), filtered to only kinds where `is_fixed_update_checker` returns
/// `true`. When both lists are empty/None, or when the filtered set is empty,
/// returns all 22 `FixedUpdate`-batch kinds as a fallback.
pub(crate) fn active_invariant_kinds(definition: &ScenarioDefinition) -> HashSet<InvariantKind> {
    let mut set: HashSet<InvariantKind> = definition
        .disallowed_failures
        .iter()
        .copied()
        .filter(|k| is_fixed_update_checker(*k))
        .collect();
    if let Some(ref allowed) = definition.allowed_failures {
        set.extend(
            allowed
                .iter()
                .copied()
                .filter(|k| is_fixed_update_checker(*k)),
        );
    }
    if set.is_empty() {
        // Fallback: register all FixedUpdate checkers.
        // This preserves backward compatibility (both lists empty)
        // AND the health check for scenarios like
        // chip_offer_expected_self_test.scenario.ron where only
        // ChipOfferExpected is in the lists (non-FixedUpdate kind
        // filtered out -> empty set -> all 21 registered).
        InvariantKind::ALL
            .iter()
            .copied()
            .filter(|k| is_fixed_update_checker(*k))
            .collect()
    } else {
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // is_fixed_update_checker — classification function
    // -----------------------------------------------------------------

    /// Behavior 1: `is_fixed_update_checker` returns `true` for FixedUpdate-batch
    /// variants. Test a representative from each of the three original batches.
    #[test]
    fn is_fixed_update_checker_returns_true_for_fixed_update_kinds() {
        // checkers_a representative
        assert!(
            is_fixed_update_checker(InvariantKind::BoltInBounds),
            "BoltInBounds should be a FixedUpdate checker"
        );
        // checkers_b representative
        assert!(
            is_fixed_update_checker(InvariantKind::NoEntityLeaks),
            "NoEntityLeaks should be a FixedUpdate checker"
        );
        // checkers_c representative
        assert!(
            is_fixed_update_checker(InvariantKind::AabbMatchesEntityDimensions),
            "AabbMatchesEntityDimensions should be a FixedUpdate checker"
        );
    }

    /// Behavior 2: `is_fixed_update_checker` returns `false` for `ChipOfferExpected`.
    #[test]
    fn is_fixed_update_checker_returns_false_for_chip_offer_expected() {
        assert!(
            !is_fixed_update_checker(InvariantKind::ChipOfferExpected),
            "ChipOfferExpected runs on Update, not FixedUpdate"
        );
    }

    /// Behavior 3: Exhaustive coverage — exactly 21 variants return `true`,
    /// exactly 1 returns `false` (`ChipOfferExpected`), total = 22 = `ALL.len()`.
    #[test]
    fn is_fixed_update_checker_covers_every_invariant_kind_variant() {
        let total = InvariantKind::ALL.len();
        assert_eq!(total, 23, "expected 23 InvariantKind variants in ALL");

        let fixed_update_count = InvariantKind::ALL
            .iter()
            .filter(|k| is_fixed_update_checker(**k))
            .count();
        let non_fixed_update_count = total - fixed_update_count;

        assert_eq!(
            fixed_update_count, 22,
            "expected exactly 22 FixedUpdate checker kinds, got {fixed_update_count}"
        );
        assert_eq!(
            non_fixed_update_count, 1,
            "expected exactly 1 non-FixedUpdate kind, got {non_fixed_update_count}"
        );

        // The sole non-FixedUpdate variant must be ChipOfferExpected
        let non_fixed: Vec<_> = InvariantKind::ALL
            .iter()
            .filter(|k| !is_fixed_update_checker(**k))
            .collect();
        assert_eq!(
            non_fixed,
            vec![&InvariantKind::ChipOfferExpected],
            "the only non-FixedUpdate kind should be ChipOfferExpected"
        );
    }

    // -----------------------------------------------------------------
    // active_invariant_kinds — set computation
    // -----------------------------------------------------------------

    /// Behavior 4: Empty `disallowed_failures` and None `allowed_failures` returns
    /// all 22 `FixedUpdate` kinds.
    #[test]
    fn active_invariant_kinds_returns_all_21_when_both_lists_empty() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![],
            allowed_failures: None,
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            22,
            "expected 22 active kinds when both lists empty, got {}",
            active.len()
        );
        assert!(
            !active.contains(&InvariantKind::ChipOfferExpected),
            "fallback set must not contain ChipOfferExpected"
        );
    }

    /// Behavior 4 edge case: Empty vec with Some(vec![]) also returns all 21.
    #[test]
    fn active_invariant_kinds_returns_all_21_when_allowed_is_empty_some() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![],
            allowed_failures: Some(vec![]),
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            22,
            "expected 21 active kinds when both lists effectively empty, got {}",
            active.len()
        );
        assert!(
            !active.contains(&InvariantKind::ChipOfferExpected),
            "fallback set must not contain ChipOfferExpected"
        );
    }

    /// Behavior 5: Single `disallowed_failures` entry returns only that kind.
    #[test]
    fn active_invariant_kinds_single_disallowed_returns_only_that_kind() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![InvariantKind::NoNaN],
            allowed_failures: None,
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            1,
            "expected 1 active kind, got {}",
            active.len()
        );
        assert!(
            active.contains(&InvariantKind::NoNaN),
            "active set must contain NoNaN"
        );
    }

    /// Behavior 6: Multiple `disallowed_failures` entries return their union.
    #[test]
    fn active_invariant_kinds_multiple_disallowed_returns_union() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![
                InvariantKind::BoltInBounds,
                InvariantKind::BreakerInBounds,
                InvariantKind::NoNaN,
            ],
            allowed_failures: None,
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            3,
            "expected 3 active kinds, got {}",
            active.len()
        );
        assert!(active.contains(&InvariantKind::BoltInBounds));
        assert!(active.contains(&InvariantKind::BreakerInBounds));
        assert!(active.contains(&InvariantKind::NoNaN));
    }

    /// Behavior 7: `allowed_failures` entries are included in active set.
    /// Edge case: same entry in both lists produces no duplication.
    #[test]
    fn active_invariant_kinds_allowed_failures_included_no_duplication() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![InvariantKind::BoltInBounds],
            allowed_failures: Some(vec![InvariantKind::BoltInBounds]),
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            1,
            "expected 1 active kind (no duplication), got {}",
            active.len()
        );
        assert!(active.contains(&InvariantKind::BoltInBounds));
    }

    /// Behavior 8: Union of `disallowed_failures` and `allowed_failures`.
    #[test]
    fn active_invariant_kinds_union_of_disallowed_and_allowed() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![InvariantKind::BoltInBounds],
            allowed_failures: Some(vec![InvariantKind::NoNaN]),
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            2,
            "expected 2 active kinds, got {}",
            active.len()
        );
        assert!(active.contains(&InvariantKind::BoltInBounds));
        assert!(active.contains(&InvariantKind::NoNaN));
    }

    /// Behavior 9: `ChipOfferExpected` in `disallowed_failures` is filtered out;
    /// only non-`ChipOfferExpected` kinds remain.
    #[test]
    fn active_invariant_kinds_filters_out_chip_offer_expected() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![
                InvariantKind::ChipOfferExpected,
                InvariantKind::BoltInBounds,
            ],
            allowed_failures: None,
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            1,
            "expected 1 active kind after filtering ChipOfferExpected, got {}",
            active.len()
        );
        assert!(active.contains(&InvariantKind::BoltInBounds));
        assert!(
            !active.contains(&InvariantKind::ChipOfferExpected),
            "ChipOfferExpected must be filtered out"
        );
    }

    /// Behavior 9 edge case: `ChipOfferExpected` as the only entry triggers
    /// fallback to all 22 `FixedUpdate` kinds.
    #[test]
    fn active_invariant_kinds_chip_offer_expected_only_triggers_fallback() {
        let def = ScenarioDefinition {
            disallowed_failures: vec![InvariantKind::ChipOfferExpected],
            allowed_failures: None,
            ..Default::default()
        };
        let active = active_invariant_kinds(&def);
        assert_eq!(
            active.len(),
            22,
            "expected 21 active kinds (fallback) when only ChipOfferExpected is listed, got {}",
            active.len()
        );
        assert!(
            !active.contains(&InvariantKind::ChipOfferExpected),
            "fallback set must not contain ChipOfferExpected"
        );
    }
}
