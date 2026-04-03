//! Tests for tier structure: boss at end of each tier, HP multipliers,
//! boss HP formula, timer reduction accumulation, timer clamp, and
//! `tier_index` assignment.

use super::{super::system::generate_node_sequence, helpers::*};
use crate::state::run::definition::{NodeType, TierNodeCount};

// -- 8. Each tier ends with a Boss node --

#[test]
fn each_tier_ends_with_boss_node() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 0.9),
            make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 0.8),
        ],
        3.0,
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

// -- 9. HP multipliers applied per tier --

#[test]
fn hp_multipliers_applied_per_tier() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
            make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 1.0),
        ],
        1.0, // boss_hp_mult = 1.0 so boss hp_mult == tier hp_mult
        0.0,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    // Tier 0: indices 0, 1 (non-boss) should have hp_mult 1.0
    for i in 0..2 {
        assert!(
            (seq.assignments[i].hp_mult - 1.0).abs() < f32::EPSILON,
            "tier 0 node {i} hp_mult should be 1.0, got {}",
            seq.assignments[i].hp_mult
        );
    }

    // Tier 1: indices 3, 4 (non-boss) should have hp_mult 2.0
    for i in 3..5 {
        assert!(
            (seq.assignments[i].hp_mult - 2.0).abs() < f32::EPSILON,
            "tier 1 node {i} hp_mult should be 2.0, got {}",
            seq.assignments[i].hp_mult
        );
    }
}

// -- 10. Boss HP = tier.hp_mult * boss_hp_mult --

#[test]
fn boss_hp_mult_is_tier_hp_mult_times_boss_hp_mult() {
    let curve = make_curve(
        vec![make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 1.0)],
        3.0,
        0.0,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    let boss = seq
        .assignments
        .iter()
        .find(|a| a.node_type == NodeType::Boss)
        .expect("should have a boss node");

    let expected = 1.5 * 3.0; // 4.5
    assert!(
        (boss.hp_mult - expected).abs() < f32::EPSILON,
        "boss hp_mult should be 4.5 (1.5 * 3.0), got {}",
        boss.hp_mult
    );
}

// -- 11. Timer reduction cumulates after each boss --

#[test]
fn timer_reduction_cumulates_after_each_boss() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
        ],
        1.0,
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
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 0.15),
            make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 0.15),
        ],
        1.0,
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
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 0.9),
            make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 0.8),
        ],
        3.0,
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
