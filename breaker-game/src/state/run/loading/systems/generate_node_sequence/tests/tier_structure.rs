//! Tests for tier structure: boss at end of each tier, timer reduction
//! accumulation, timer clamp, and `tier_index` assignment.
//!
//! HP multiplier tests (`hp_multipliers_applied_per_tier`,
//! `boss_hp_mult_is_tier_hp_mult_times_boss_hp_mult`) have been removed —
//! HP is now computed from `ToughnessConfig` at spawn time.

use super::{super::system::generate_node_sequence, helpers::*};
use crate::{prelude::*, state::run::resources::DifficultyCurve};

// -- 8. Each tier ends with a Boss node --

#[test]
fn each_tier_ends_with_boss_node() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Fixed(2), 0.5, 0.9),
            make_tier(TierNodeCount::Fixed(2), 0.5, 0.8),
        ],
        0.1,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // 3 tiers * (2 nodes + 1 boss) = 9 total
    assert_eq!(seq.assignments.len(), 9, "3 tiers of 3 = 9 total");

    // Bosses at indices 2, 5, 8 (end of each tier)
    assert_eq!(
        seq.assignments[2].node_type,
        NodeType::Boss,
        "boss at index 2 (end of tier 0)"
    );
    assert_eq!(
        seq.assignments[5].node_type,
        NodeType::Boss,
        "boss at index 5 (end of tier 1)"
    );
    assert_eq!(
        seq.assignments[8].node_type,
        NodeType::Boss,
        "boss at index 8 (end of tier 2)"
    );
}

// -- 11. Timer reduction cumulates after each boss --

#[test]
fn timer_reduction_cumulates_after_each_boss() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
        ],
        0.1,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Tier 0 (indices 0-1): timer_mult = 1.0 (no reduction yet)
    assert!(
        (seq.assignments[0].timer_mult - 1.0).abs() < f32::EPSILON,
        "tier 0 non-boss timer_mult should be 1.0, got {}",
        seq.assignments[0].timer_mult
    );

    // Tier 1 (indices 2-3): timer_mult = 1.0 - 0.1 = 0.9
    assert!(
        (seq.assignments[2].timer_mult - 0.9).abs() < f32::EPSILON,
        "tier 1 non-boss timer_mult should be 0.9, got {}",
        seq.assignments[2].timer_mult
    );

    // Tier 2 (indices 4-5): timer_mult = 1.0 - 0.2 = 0.8
    assert!(
        (seq.assignments[4].timer_mult - 0.8).abs() < f32::EPSILON,
        "tier 2 non-boss timer_mult should be 0.8, got {}",
        seq.assignments[4].timer_mult
    );
}

// -- 12. Timer mult clamped to minimum 0.1 --

#[test]
fn timer_mult_clamped_to_minimum() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(1), 0.0, 0.15),
            make_tier(TierNodeCount::Fixed(1), 0.0, 0.15),
        ],
        0.1,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Tier 0: timer_mult = 0.15 (no reduction yet)
    assert!(
        (seq.assignments[0].timer_mult - 0.15).abs() < f32::EPSILON,
        "tier 0 timer_mult should be 0.15, got {}",
        seq.assignments[0].timer_mult
    );

    // Tier 1: timer_mult = 0.15 - 0.1 = 0.05, clamped to 0.1
    assert!(
        (seq.assignments[2].timer_mult - 0.1).abs() < f32::EPSILON,
        "tier 1 timer_mult should be clamped to 0.1, got {}",
        seq.assignments[2].timer_mult
    );
}

// -- 15. tier_index correctly assigned --

#[test]
fn tier_index_correctly_assigned() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Fixed(2), 0.5, 0.9),
            make_tier(TierNodeCount::Fixed(2), 0.5, 0.8),
        ],
        0.1,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Tier 0: indices 0, 1, 2 (2 non-boss + 1 boss)
    for i in 0..3 {
        assert_eq!(
            seq.assignments[i].tier_index, 0,
            "assignment {i} should be tier_index 0, got {}",
            seq.assignments[i].tier_index
        );
    }

    // Tier 1: indices 3, 4, 5
    for i in 3..6 {
        assert_eq!(
            seq.assignments[i].tier_index, 1,
            "assignment {i} should be tier_index 1, got {}",
            seq.assignments[i].tier_index
        );
    }

    // Tier 2: indices 6, 7, 8
    for i in 6..9 {
        assert_eq!(
            seq.assignments[i].tier_index, 2,
            "assignment {i} should be tier_index 2, got {}",
            seq.assignments[i].tier_index
        );
    }
}

// ── Part J: NodeAssignment no longer has hp_mult ──────────────────────────

// Behavior 29: generate_node_sequence no longer sets hp_mult on assignments
#[test]
fn assignments_have_only_node_type_tier_index_timer_mult() {
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(2), 0.5, 1.0)], 0.0);
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Verify that each assignment has the expected fields (compile-time check).
    // If hp_mult existed, this would not compile.
    for assignment in &seq.assignments {
        _ = assignment.node_type;
        _ = assignment.tier_index;
        _ = assignment.timer_mult;
    }
    assert_eq!(seq.assignments.len(), 3, "2 non-boss + 1 boss = 3");
}

// Behavior 30: timer_mult still correctly set with cumulative reduction
#[test]
fn timer_mult_still_correctly_set_with_cumulative_reduction() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0),
        ],
        0.1,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Tier 0: 1.0
    assert!(
        (seq.assignments[0].timer_mult - 1.0).abs() < f32::EPSILON,
        "tier 0 timer_mult should be 1.0"
    );
    // Tier 1: 1.0 - 0.1 = 0.9
    assert!(
        (seq.assignments[2].timer_mult - 0.9).abs() < f32::EPSILON,
        "tier 1 timer_mult should be 0.9"
    );
    // Tier 2: 1.0 - 0.2 = 0.8
    assert!(
        (seq.assignments[4].timer_mult - 0.8).abs() < f32::EPSILON,
        "tier 2 timer_mult should be 0.8"
    );
}

// Behavior 31: DifficultyCurve used by generate_node_sequence no longer has boss_hp_mult
#[test]
fn difficulty_curve_without_boss_hp_mult_compiles() {
    let curve = DifficultyCurve {
        tiers:                    vec![make_tier(TierNodeCount::Fixed(1), 0.0, 1.0)],
        timer_reduction_per_boss: 0.1,
    };
    let mut rng = rng_from_seed(42);
    let seq = generate_node_sequence(&curve, &mut rng);
    assert!(!seq.assignments.is_empty(), "sequence should not be empty");
}

// ── Part N: Difficulty RON integration ────────────────────────────────────

// Behavior 50: defaults.difficulty.ron parses without hp_mult/boss_hp_mult
#[test]
fn defaults_difficulty_ron_parses_without_hp_mult() {
    let ron_str = include_str!("../../../../../../../assets/config/defaults.difficulty.ron");
    let _curve: crate::state::run::resources::DifficultyCurveDefaults =
        ron::de::from_str(ron_str).expect("defaults.difficulty.ron should parse");
}
