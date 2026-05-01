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

use super::super::{
    debug_setup::{
        apply_debug_setup, deferred_debug_setup, enforce_frozen_positions, enforce_frozen_velocity,
    },
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
        check_burnout_heat_clamped, check_burnout_state_orphaned, check_chain_arc_count_reasonable,
        check_chip_offer_expected, check_chip_stacks_consistent, check_debt_collector_orphaned,
        check_echo_strike_orphaned, check_exactly_one_primary_bolt, check_fission_counter_orphaned,
        check_gravity_well_count_reasonable, check_greed_stacks_orphaned, check_hazard_stack_valid,
        check_maxed_chip_never_offered, check_no_entity_leaks, check_no_nan,
        check_offering_no_duplicates, check_pulse_ring_accumulation, check_reckless_dash_orphaned,
        check_run_stats_monotonic, check_second_wind_wall_at_most_one,
        check_shield_wall_at_most_one, check_siphon_streak_orphaned,
        check_timer_monotonically_decreasing, check_timer_non_negative, check_valid_breaker_state,
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
        .add_message::<breaker::state::run::node::messages::IncreaseNodeTimer>();
}

/// Returns `true` when the scenario has entered the `Playing` phase,
/// gating invariant checkers until entities are fully initialised.
fn playing_gate(stats: Option<Res<ScenarioStats>>) -> bool {
    stats.is_some_and(|s| s.entered_playing)
}

/// Stricter gate for invariants whose contract only holds while
/// `NodeState::Playing` is the current state. Requires both that Playing
/// has been entered at least once (`playing_gate`) AND that the current
/// state is still Playing.
///
/// `BoltSpeedAccurate` uses this stricter gate because its game-side
/// counterpart `sync_bolt_speed_to_stack` is also gated to
/// `in_state(NodeState::Playing)`. Without matching gates, an
/// `Until(NodeEndOccurred, Fire(SpeedBoost))` reversal during
/// `OnEnter(NodeState::Teardown)` clears the stack but leaves the bolt
/// velocity unchanged until the next Playing tick — a transient mismatch
/// the invariant would otherwise flag during the gap between teardown and
/// the next node's first `FixedUpdate` tick.
///
/// Both parameters use `Option<Res<...>>` so the gate is well-defined in
/// minimal test apps: returns `false` if either resource is absent. In
/// the full scenario lifecycle both are always present.
pub(crate) fn playing_state_gate(
    stats: Option<Res<ScenarioStats>>,
    node_state: Option<Res<State<NodeState>>>,
) -> bool {
    stats.is_some_and(|s| s.entered_playing)
        && node_state.is_some_and(|c| matches!(*c.get(), NodeState::Playing))
}

/// Registers each `FixedUpdate` invariant checker that is in the active set.
///
/// Split into core (engine-level) and protocol-orphan helpers to keep each
/// function under the 100-line clippy pedantic limit. The macro is defined
/// in each helper because it captures `app` and `active` by name.
fn register_active_checkers(app: &mut App, active: &HashSet<InvariantKind>) {
    register_core_invariant_checkers(app, active);
    register_protocol_orphan_checkers(app, active);
}

/// Registers the engine-level invariant checkers (bolt/breaker/chip/effect).
///
/// Uses a macro to avoid repeating the identical ordering constraints
/// (`.run_if(playing_gate).after(...).before(...)`) for each checker.
fn register_core_invariant_checkers(app: &mut App, active: &HashSet<InvariantKind>) {
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
    // BoltSpeedAccurate needs to run AFTER `BoltSystems::SyncSpeedToStack` so
    // it observes the post-sync velocity. Note: it cannot also be
    // `.before(BoltSystems::BoltLost)` because `bolt_lost` is ordered
    // `.before(EffectV3Systems::Bridge)` which transitively runs before
    // `EffectV3Systems::Death` (and thus before `SyncSpeedToStack`). Adding
    // `.before(BoltLost)` would create a cycle. Without that edge, the
    // checker simply runs once SyncSpeedToStack has flushed — sufficient for
    // accurate observation (W8 §A regression).
    if active.contains(&InvariantKind::BoltSpeedAccurate) {
        app.add_systems(
            FixedUpdate,
            check_bolt_speed_accurate
                .run_if(playing_state_gate)
                .after(apply_debug_frame_mutations)
                .after(deferred_debug_setup)
                .after(tag_game_entities)
                .after(BreakerSystems::UpdateState)
                .after(BoltSystems::SyncSpeedToStack)
                .after(enforce_frozen_velocity),
        );
    }
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
    register_checker!(InvariantKind::HazardStackValid, check_hazard_stack_valid);
    register_checker!(
        InvariantKind::ExactlyOnePrimaryBolt,
        check_exactly_one_primary_bolt
    );
}

/// Registers the protocol-owned invariant checkers (ownership/lifecycle).
///
/// Each checker pins a CONTRACT that per-protocol state is cleared when the
/// protocol is removed from `ActiveProtocols`. See
/// `docs/architecture/scenario-runner.md` for the contract-vs-clamp
/// distinction.
fn register_protocol_orphan_checkers(app: &mut App, active: &HashSet<InvariantKind>) {
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

    register_checker!(
        InvariantKind::BurnoutHeatClamped,
        check_burnout_heat_clamped
    );
    register_checker!(
        InvariantKind::GreedStacksOrphaned,
        check_greed_stacks_orphaned
    );
    register_checker!(
        InvariantKind::SiphonStreakOrphaned,
        check_siphon_streak_orphaned
    );
    register_checker!(
        InvariantKind::FissionCounterOrphaned,
        check_fission_counter_orphaned
    );
    register_checker!(
        InvariantKind::RecklessDashOrphaned,
        check_reckless_dash_orphaned
    );
    register_checker!(
        InvariantKind::EchoStrikeOrphaned,
        check_echo_strike_orphaned
    );
    register_checker!(
        InvariantKind::DebtCollectorOrphaned,
        check_debt_collector_orphaned
    );
    register_checker!(
        InvariantKind::BurnoutStateOrphaned,
        check_burnout_state_orphaned
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

    // `enforce_frozen_velocity` MUST NOT chain into `BoltSystems::BoltLost`
    // (that path closes a cycle: BoltLost → Bridge → ApplyDeferred →
    // SyncSpeedToStack → enforce_frozen_velocity). It only needs to run
    // after `SyncSpeedToStack` re-normalizes velocity, so the per-tick
    // re-pin happens AFTER the sync overwrites it.
    app.add_systems(
        FixedUpdate,
        enforce_frozen_velocity
            .run_if(playing_gate)
            .after(BoltSystems::SyncSpeedToStack),
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
/// Uses an exhaustive match so new variants produce a compile error (33 true,
/// 1 false — `ChipOfferExpected`).
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
        | InvariantKind::BoltBirthingLayersZeroed
        | InvariantKind::HazardStackValid
        | InvariantKind::ExactlyOnePrimaryBolt
        | InvariantKind::BurnoutHeatClamped
        | InvariantKind::GreedStacksOrphaned
        | InvariantKind::SiphonStreakOrphaned
        | InvariantKind::FissionCounterOrphaned
        | InvariantKind::RecklessDashOrphaned
        | InvariantKind::EchoStrikeOrphaned
        | InvariantKind::DebtCollectorOrphaned
        | InvariantKind::BurnoutStateOrphaned => true,
        InvariantKind::ChipOfferExpected => false,
    }
}

/// Computes the set of `InvariantKind` variants that should have their
/// `FixedUpdate` checkers registered for this scenario.
///
/// Takes the union of `disallowed_failures` and `allowed_failures` (if
/// present), filtered to only kinds where `is_fixed_update_checker` returns
/// `true`. When both lists are empty/None, or when the filtered set is empty,
/// returns all 33 `FixedUpdate`-batch kinds as a fallback.
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
        // filtered out -> empty set -> all 33 registered).
        InvariantKind::ALL
            .iter()
            .copied()
            .filter(|k| is_fixed_update_checker(*k))
            .collect()
    } else {
        set
    }
}
