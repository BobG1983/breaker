//! Tests for node count behavior: `Fixed(N)`, `Range(min, max)`,
//! empty tiers, and `Fixed(0)`.

use super::{super::system::generate_node_sequence, helpers::*};
use crate::run::definition::{NodeType, TierNodeCount};

// -- 3. Fixed(5) produces 5 non-boss + 1 boss per tier --

#[test]
fn fixed_count_produces_correct_node_totals() {
    let curve = make_curve(
        vec![make_tier(TierNodeCount::Fixed(5), 0.5, 1.0, 1.0)],
        3.0,
        0.0,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    assert_eq!(
        seq.assignments.len(),
        6,
        "Fixed(5) + boss = 6 total assignments"
    );

    let non_boss_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type != NodeType::Boss)
        .count();
    assert_eq!(non_boss_count, 5, "should have 5 non-boss nodes");

    assert_eq!(
        seq.assignments.last().unwrap().node_type,
        NodeType::Boss,
        "last node in tier must be Boss"
    );
}

// -- 4. Range(4, 6) stays in bounds across 100 seeds --

#[test]
fn range_count_stays_within_bounds_across_seeds() {
    let curve = make_curve(
        vec![make_tier(TierNodeCount::Range(4, 6), 0.5, 1.0, 1.0)],
        3.0,
        0.0,
    );

    for seed in 0..100_u64 {
        let mut rng = rng_from_seed(seed);
        let seq = generate_node_sequence(&curve, &mut rng);
        let len = seq.assignments.len();
        assert!(
            (5..=7).contains(&len),
            "seed {seed}: expected 5..=7 assignments (4..=6 + boss), got {len}"
        );
    }
}

// -- 13. Empty tiers produces empty sequence --

#[test]
fn empty_tiers_produces_empty_sequence() {
    let curve = make_curve(vec![], 3.0, 0.1);
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    assert!(
        seq.assignments.is_empty(),
        "no tiers should produce empty sequence"
    );
}

// -- 14. Fixed(0) produces only Boss node --

#[test]
fn fixed_zero_produces_only_boss_node() {
    let curve = make_curve(
        vec![make_tier(TierNodeCount::Fixed(0), 0.5, 1.0, 1.0)],
        3.0,
        0.0,
    );
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    assert_eq!(
        seq.assignments.len(),
        1,
        "Fixed(0) should produce 1 assignment (boss only)"
    );
    assert_eq!(
        seq.assignments[0].node_type,
        NodeType::Boss,
        "the single assignment must be a Boss"
    );
}
