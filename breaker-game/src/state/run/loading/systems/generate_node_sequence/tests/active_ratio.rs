//! Tests for `active_ratio` behavior: 0.0 (all Passive), 1.0 (all Active),
//! and fractional rounding.

use super::{super::system::generate_node_sequence, helpers::*};
use crate::state::run::definition::{NodeType, TierNodeCount};

// -- 5. active_ratio 0.0 produces all Passive --

#[test]
fn active_ratio_zero_produces_all_passive() {
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(5), 0.0, 1.0)], 0.0);
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    let active_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Active)
        .count();
    let passive_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Passive)
        .count();
    let boss_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Boss)
        .count();

    assert_eq!(active_count, 0, "active_ratio 0.0 should produce 0 Active");
    assert_eq!(passive_count, 5, "should have 5 Passive nodes");
    assert_eq!(boss_count, 1, "should have 1 Boss node");
}

// -- 6. active_ratio 1.0 produces all Active --

#[test]
fn active_ratio_one_produces_all_active() {
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(5), 1.0, 1.0)], 0.0);
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    let active_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Active)
        .count();
    let passive_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Passive)
        .count();
    let boss_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Boss)
        .count();

    assert_eq!(active_count, 5, "active_ratio 1.0 should produce 5 Active");
    assert_eq!(passive_count, 0, "should have 0 Passive nodes");
    assert_eq!(boss_count, 1, "should have 1 Boss node");
}

// -- 7. active_ratio 0.4 with Fixed(5) produces 2 Active + 3 Passive --

#[test]
fn active_ratio_fractional_rounds_correctly() {
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(5), 0.4, 1.0)], 0.0);
    let mut rng = rng_from_seed(42);

    let seq = generate_node_sequence(&curve, &mut rng);

    let active_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Active)
        .count();
    let passive_count = seq
        .assignments
        .iter()
        .filter(|a| a.node_type == NodeType::Passive)
        .count();

    assert_eq!(active_count, 2, "round(5 * 0.4) = 2 Active nodes");
    assert_eq!(passive_count, 3, "5 - 2 = 3 Passive nodes");
}
