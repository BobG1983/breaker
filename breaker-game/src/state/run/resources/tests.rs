use bevy::prelude::*;

use super::definitions::*;
use crate::{
    mutators::hazards::definition::types::HazardKind,
    state::run::{
        definition::{TierDefinition, TierNodeCount},
        generation::types::TierModifierPool,
    },
};

#[test]
fn default_run_state_starts_at_node_zero() {
    let state = NodeOutcome::default();
    assert_eq!(state.node_index, 0);
}

#[test]
fn default_outcome_is_in_progress() {
    let state = NodeOutcome::default();
    assert_eq!(state.result, NodeResult::InProgress);
}

// -- DifficultyCurve From conversion --

#[test]
fn difficulty_curve_from_defaults_copies_all_fields() {
    let defaults = DifficultyCurveDefaults {
        tiers:                    vec![
            TierDefinition {
                nodes:            TierNodeCount::Fixed(3),
                active_ratio:     0.0,
                timer_mult:       1.0,
                introduced_cells: vec![],
            },
            TierDefinition {
                nodes:            TierNodeCount::Range(4, 6),
                active_ratio:     0.5,
                timer_mult:       0.8,
                introduced_cells: vec!['T'],
            },
        ],
        timer_reduction_per_boss: 0.1,
    };

    let curve = DifficultyCurve::from(defaults);

    assert_eq!(curve.tiers.len(), 2, "tier count should match");
    assert!(
        (curve.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON,
        "timer_reduction_per_boss should be 0.1, got {}",
        curve.timer_reduction_per_boss
    );
    // Spot-check first tier fields
    assert!(
        (curve.tiers[0].active_ratio - 0.0).abs() < f32::EPSILON,
        "first tier active_ratio should be 0.0"
    );
}

// -- flux_earned calculation --

#[test]
fn flux_earned_with_concrete_values() {
    let stats = RunStats {
        nodes_cleared: 5,
        perfect_bumps: 10,
        evolutions_performed: 1,
        bolts_lost: 3,
        ..Default::default()
    };
    // (5*10) + (10*2) + (1*25) - (3*3) = 50 + 20 + 25 - 9 = 86
    assert_eq!(
        stats.flux_earned(),
        86,
        "flux = (5*10) + (10*2) + (1*25) - (3*3) = 86"
    );
}

// -- HighlightKind::category mapping --

#[test]
fn mass_destruction_maps_to_execution() {
    assert_eq!(
        HighlightKind::MassDestruction.category(),
        HighlightCategory::Execution
    );
}

#[test]
fn combo_king_maps_to_execution() {
    assert_eq!(
        HighlightKind::ComboKing.category(),
        HighlightCategory::Execution
    );
}

#[test]
fn pinball_wizard_maps_to_execution() {
    assert_eq!(
        HighlightKind::PinballWizard.category(),
        HighlightCategory::Execution
    );
}

#[test]
fn perfect_streak_maps_to_execution() {
    assert_eq!(
        HighlightKind::PerfectStreak.category(),
        HighlightCategory::Execution
    );
}

#[test]
fn perfect_node_maps_to_execution() {
    assert_eq!(
        HighlightKind::PerfectNode.category(),
        HighlightCategory::Execution
    );
}

#[test]
fn no_damage_node_maps_to_endurance() {
    assert_eq!(
        HighlightKind::NoDamageNode.category(),
        HighlightCategory::Endurance
    );
}

#[test]
fn untouchable_maps_to_endurance() {
    assert_eq!(
        HighlightKind::Untouchable.category(),
        HighlightCategory::Endurance
    );
}

#[test]
fn comeback_maps_to_endurance() {
    assert_eq!(
        HighlightKind::Comeback.category(),
        HighlightCategory::Endurance
    );
}

#[test]
fn first_evolution_maps_to_progression() {
    assert_eq!(
        HighlightKind::FirstEvolution.category(),
        HighlightCategory::Progression
    );
}

#[test]
fn most_powerful_evolution_maps_to_progression() {
    assert_eq!(
        HighlightKind::MostPowerfulEvolution.category(),
        HighlightCategory::Progression
    );
}

#[test]
fn clutch_clear_maps_to_clutch() {
    assert_eq!(
        HighlightKind::ClutchClear.category(),
        HighlightCategory::Clutch
    );
}

#[test]
fn fast_clear_maps_to_clutch() {
    assert_eq!(
        HighlightKind::FastClear.category(),
        HighlightCategory::Clutch
    );
}

#[test]
fn speed_demon_maps_to_clutch() {
    assert_eq!(
        HighlightKind::SpeedDemon.category(),
        HighlightCategory::Clutch
    );
}

#[test]
fn close_save_maps_to_clutch() {
    assert_eq!(
        HighlightKind::CloseSave.category(),
        HighlightCategory::Clutch
    );
}

#[test]
fn nail_biter_maps_to_clutch() {
    assert_eq!(
        HighlightKind::NailBiter.category(),
        HighlightCategory::Clutch
    );
}

// -- flux_earned calculation --

#[test]
fn flux_earned_floors_at_zero_when_penalty_exceeds_bonuses() {
    let stats = RunStats {
        nodes_cleared: 0,
        perfect_bumps: 0,
        evolutions_performed: 0,
        bolts_lost: 10,
        ..Default::default()
    };
    assert_eq!(
        stats.flux_earned(),
        0,
        "flux should floor at 0, not go negative"
    );
}

// ── Wave 1B Group A — RunProgress resource shape and defaults ──────────────

// Behavior 1: RunProgress::default() zeroes every field
#[test]
fn run_progress_default_zeroes_all_fields() {
    let progress = RunProgress::default();
    assert_eq!(progress.tier_index, 0);
    assert_eq!(progress.node_in_tier, 0);
    assert_eq!(progress.total_nodes_cleared, 0);
}

// Behavior 1 edge case: explicit constructor equals default (field-by-field)
#[test]
fn run_progress_explicit_zero_equals_default_fields() {
    let explicit = RunProgress {
        tier_index:          0,
        node_in_tier:        0,
        total_nodes_cleared: 0,
    };
    let default = RunProgress::default();
    assert_eq!(explicit.tier_index, default.tier_index);
    assert_eq!(explicit.node_in_tier, default.node_in_tier);
    assert_eq!(explicit.total_nodes_cleared, default.total_nodes_cleared);
}

// Behavior 2: RunProgress can be constructed with explicit field values
#[test]
fn run_progress_explicit_fields_round_trip() {
    let progress = RunProgress {
        tier_index:          3,
        node_in_tier:        4,
        total_nodes_cleared: 17,
    };
    assert_eq!(progress.tier_index, 3);
    assert_eq!(progress.node_in_tier, 4);
    assert_eq!(progress.total_nodes_cleared, 17);
}

// Behavior 2 edge case: maximum u32 values round-trip unchanged
#[test]
fn run_progress_max_values_round_trip() {
    let progress = RunProgress {
        tier_index:          u32::MAX,
        node_in_tier:        u32::MAX,
        total_nodes_cleared: u32::MAX,
    };
    assert_eq!(progress.tier_index, u32::MAX);
    assert_eq!(progress.node_in_tier, u32::MAX);
    assert_eq!(progress.total_nodes_cleared, u32::MAX);
}

// Behavior 3: RunProgress is a Bevy Resource — init_resource inserts default
#[test]
fn run_progress_is_resource_init_inserts_default() {
    let mut app = App::new();
    app.init_resource::<RunProgress>();
    let res = app
        .world()
        .get_resource::<RunProgress>()
        .expect("RunProgress must be present after init_resource");
    assert_eq!(res.tier_index, 0);
    assert_eq!(res.node_in_tier, 0);
    assert_eq!(res.total_nodes_cleared, 0);
}

// Behavior 3 edge case: panicking accessor does not panic
#[test]
fn run_progress_resource_panicking_accessor_does_not_panic() {
    let mut app = App::new();
    app.init_resource::<RunProgress>();
    let _ = app.world().resource::<RunProgress>();
}

// ── Wave 1B Group B — TierConfig resource shape and defaults ──────────────

// Behavior 4: TierConfig::default() produces zeroed tier, None handle, empty stack
#[test]
fn tier_config_default_zeroed_fields() {
    let cfg = TierConfig::default();
    assert_eq!(cfg.tier_index, 0);
    assert!(cfg.modifier_pool_handle.is_none());
    assert!(cfg.hazard_stack.is_empty());
}

// Behavior 4 edge case: hazard_stack capacity unconstrained — only emptiness matters
#[test]
fn tier_config_default_hazard_stack_is_empty_not_capacity_checked() {
    let cfg = TierConfig::default();
    assert!(
        cfg.hazard_stack.is_empty(),
        "hazard_stack must be empty on default; capacity is allowed to be any value"
    );
}

// Behavior 5: TierConfig can hold Some(Handle<TierModifierPool>)
// TierModifierPool has no Default impl yet — use Handle::default() (invalid handle, type-correct)
#[test]
fn tier_config_modifier_pool_handle_accepts_some() {
    let handle: Handle<TierModifierPool> = Handle::default();
    let cfg = TierConfig {
        modifier_pool_handle: Some(handle),
        ..Default::default()
    };
    assert!(
        cfg.modifier_pool_handle.is_some(),
        "modifier_pool_handle must be Some after assignment"
    );
}

// Behavior 5 edge case: setting back to None clears the field
#[test]
fn tier_config_modifier_pool_handle_can_be_set_to_none() {
    let handle: Handle<TierModifierPool> = Handle::default();
    let mut cfg = TierConfig {
        modifier_pool_handle: Some(handle),
        ..Default::default()
    };
    assert!(cfg.modifier_pool_handle.is_some());
    cfg.modifier_pool_handle = None;
    assert!(
        cfg.modifier_pool_handle.is_none(),
        "modifier_pool_handle must be None after reassignment to None"
    );
}

// Behavior 6: hazard_stack accepts pushed HazardKind variants
#[test]
fn tier_config_hazard_stack_accepts_pushed_variants() {
    let mut cfg = TierConfig::default();
    cfg.hazard_stack.push(HazardKind::Decay);
    cfg.hazard_stack.push(HazardKind::Drift);
    assert_eq!(cfg.hazard_stack.len(), 2);
    assert_eq!(cfg.hazard_stack[0], HazardKind::Decay);
    assert_eq!(cfg.hazard_stack[1], HazardKind::Drift);
}

// Behavior 6 edge case: pushing same variant twice produces len == 2, no dedup
#[test]
fn tier_config_hazard_stack_no_dedup_on_duplicate_push() {
    let mut cfg = TierConfig::default();
    cfg.hazard_stack.push(HazardKind::Decay);
    cfg.hazard_stack.push(HazardKind::Decay);
    assert_eq!(cfg.hazard_stack.len(), 2);
    assert_eq!(cfg.hazard_stack[0], HazardKind::Decay);
    assert_eq!(cfg.hazard_stack[1], HazardKind::Decay);
}

// Behavior 7: TierConfig is a Bevy Resource — init_resource does not require Assets<TierModifierPool>
#[test]
fn tier_config_is_resource_init_inserts_default() {
    let mut app = App::new();
    app.init_resource::<TierConfig>();
    let res = app
        .world()
        .get_resource::<TierConfig>()
        .expect("TierConfig must be present after init_resource");
    assert_eq!(res.tier_index, 0);
    assert!(res.modifier_pool_handle.is_none());
    assert!(res.hazard_stack.is_empty());
}

// Behavior 7 edge case: panicking accessor does not panic
#[test]
fn tier_config_resource_panicking_accessor_does_not_panic() {
    let mut app = App::new();
    app.init_resource::<TierConfig>();
    let _ = app.world().resource::<TierConfig>();
}
