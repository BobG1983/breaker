//! Tests for determinism: same seed produces identical sequences,
//! different seeds produce different sequences.

use super::{super::system::generate_node_sequence, helpers::*};
use crate::prelude::*;

// -- 1. Determinism: same seed produces identical sequence --

#[test]
fn same_seed_produces_identical_sequence() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(3), 0.5, 1.0),
            make_tier(TierNodeCount::Fixed(3), 0.5, 0.9),
        ],
        0.1,
    );

    let mut rng1 = rng_from_seed(42);
    let mut rng2 = rng_from_seed(42);

    let seq1 = generate_node_sequence(&curve, &mut rng1);
    let seq2 = generate_node_sequence(&curve, &mut rng2);

    assert_eq!(
        seq1.assignments, seq2.assignments,
        "same seed must produce identical node sequences"
    );
    // Also verify non-empty (stub returns empty, so this forces implementation)
    assert!(
        !seq1.assignments.is_empty(),
        "sequence must not be empty for a curve with tiers"
    );
}

// -- 2. Different seeds produce different sequences --

#[test]
fn different_seeds_produce_different_sequences() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(3), 0.5, 1.0),
            make_tier(TierNodeCount::Fixed(3), 0.5, 0.9),
        ],
        0.1,
    );

    let mut rng_a = rng_from_seed(42);
    let mut rng_b = rng_from_seed(99);

    let seq_a = generate_node_sequence(&curve, &mut rng_a);
    let seq_b = generate_node_sequence(&curve, &mut rng_b);

    assert_ne!(
        seq_a.assignments, seq_b.assignments,
        "different seeds should produce different sequences (shuffle differs)"
    );
}
