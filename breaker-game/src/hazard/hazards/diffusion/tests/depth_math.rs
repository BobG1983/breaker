//! Section A — `DiffusionConfig::depth` pure formula.

use super::{super::system::*, helpers::*};

// Behavior 9 — depth(0) == 0 (inactive short-circuit).
#[test]
fn depth_zero_stacks_is_zero() {
    let cfg = canonical_config();
    assert_eq!(cfg.depth(0), 0);
}

// Behavior 10 — depth(1..=5) with interval 5 == 1 for every stack in range.
#[test]
fn depth_stack_one_through_five_is_one() {
    let cfg = canonical_config();
    for k in 1..=5u32 {
        assert_eq!(cfg.depth(k), 1, "stack {k} should have depth 1");
    }
}

#[test]
fn depth_stack_five_boundary_is_one_not_two() {
    // Pins integer-division boundary: 1 + (5 - 1) / 5 == 1 + 0 == 1.
    let cfg = canonical_config();
    assert_eq!(cfg.depth(5), 1);
}

// Behavior 11 — depth(6..=10) with interval 5 == 2.
#[test]
fn depth_stack_six_is_two() {
    let cfg = canonical_config();
    assert_eq!(cfg.depth(6), 2);
}

#[test]
fn depth_stack_ten_is_two() {
    let cfg = canonical_config();
    assert_eq!(cfg.depth(10), 2);
}

// Behavior 12 — depth(11) with interval 5 == 3.
#[test]
fn depth_stack_eleven_is_three() {
    let cfg = canonical_config();
    assert_eq!(cfg.depth(11), 3);
}

// Behavior 13 — depth(u32::MAX) saturates and does not panic.
#[test]
fn depth_u32_max_does_not_panic_and_is_at_least_one() {
    let cfg = canonical_config();
    let d = cfg.depth(u32::MAX);
    assert!(d >= 1, "depth(u32::MAX) should be >= 1, got {d}");
}

// Behavior 14 — depth with depth_increase_interval = 0 returns 1 regardless.
#[test]
fn depth_with_zero_interval_returns_one_at_stack_one() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 0,
    };
    assert_eq!(cfg.depth(1), 1);
}

#[test]
fn depth_with_zero_interval_returns_one_at_stack_one_hundred() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 0,
    };
    assert_eq!(cfg.depth(100), 1);
}
