use super::definitions::*;
use crate::state::run::definition::{TierDefinition, TierNodeCount};

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
        tiers: vec![
            TierDefinition {
                nodes: TierNodeCount::Fixed(3),
                active_ratio: 0.0,
                timer_mult: 1.0,
                introduced_cells: vec![],
            },
            TierDefinition {
                nodes: TierNodeCount::Range(4, 6),
                active_ratio: 0.5,
                timer_mult: 0.8,
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
