//! Section A — `DiffusionConfig::share_percent` pure formula.

use super::{super::system::*, helpers::*};

// Behavior 1 — share_percent(0) == 0.0 (inactive short-circuit).
#[test]
fn share_percent_zero_stacks_is_zero() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 5,
    };
    assert!((cfg.share_percent(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn share_percent_zero_stacks_short_circuits_before_cap() {
    let cfg = DiffusionConfig {
        base_share_percent:      95.0,
        share_per_level_percent: 0.0,
        depth_increase_interval: 5,
    };
    // Pins that stacks == 0 returns 0.0 BEFORE cap logic applies.
    assert!((cfg.share_percent(0) - 0.0).abs() < f32::EPSILON);
}

// Behavior 2 — share_percent(1) == base_share_percent.
#[test]
fn share_percent_stack_one_is_base() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 5,
    };
    assert!((cfg.share_percent(1) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn share_percent_stack_one_ignores_per_level_multiplier() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 999.0,
        depth_increase_interval: 5,
    };
    // per_level must not leak into stack 1 (pins extra = stacks - 1 = 0).
    assert!((cfg.share_percent(1) - 20.0).abs() < f32::EPSILON);
}

// Behavior 3 — share_percent(3) == base + 2 * per_level.
#[test]
fn share_percent_stack_three_adds_two_levels() {
    let cfg = DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 5,
    };
    // 20 + 2 * 10 = 40.
    assert!((cfg.share_percent(3) - 40.0).abs() < f32::EPSILON);
}

#[test]
fn share_percent_stack_three_with_zero_base_still_grows() {
    let cfg = DiffusionConfig {
        base_share_percent:      0.0,
        share_per_level_percent: 20.0,
        depth_increase_interval: 5,
    };
    assert!((cfg.share_percent(3) - 40.0).abs() < f32::EPSILON);
}

// Behavior 4 — share_percent(5) == 60.0 (design doc example).
#[test]
fn share_percent_stack_five_is_sixty() {
    let cfg = canonical_config();
    assert!((cfg.share_percent(5) - 60.0).abs() < f32::EPSILON);
}

// Behavior 5 — share_percent(6) == 70.0 (design doc example, also depth=2).
#[test]
fn share_percent_stack_six_is_seventy() {
    let cfg = canonical_config();
    assert!((cfg.share_percent(6) - 70.0).abs() < f32::EPSILON);
}

// Behavior 6 — share_percent capped at DIFFUSION_SHARE_CAP_PERCENT (95.0).
#[test]
fn share_percent_stack_nine_is_capped_at_ninety_five() {
    let cfg = canonical_config();
    // Raw would be 20 + 8 * 10 = 100.0, capped to 95.0.
    assert!((cfg.share_percent(9) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
}

#[test]
fn share_percent_cap_engages_at_large_stack_counts() {
    let cfg = canonical_config();
    // Raw would be 20 + 99 * 10 = 1010.0, capped to 95.0.
    assert!((cfg.share_percent(100) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
}

// Behavior 7 — cap applies to crushing base configs too.
#[test]
fn share_percent_cap_applies_post_sum_to_large_base() {
    let cfg = DiffusionConfig {
        base_share_percent:      99.0,
        share_per_level_percent: 0.0,
        depth_increase_interval: 5,
    };
    // 99.0 > 95.0; result must be the cap.
    assert!((cfg.share_percent(1) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
}

// Behavior 8 — inclusive-boundary: exactly 95.0 returns 95.0 unchanged.
#[test]
fn share_percent_exactly_at_cap_returns_cap() {
    let cfg = DiffusionConfig {
        base_share_percent:      95.0,
        share_per_level_percent: 0.0,
        depth_increase_interval: 5,
    };
    assert!((cfg.share_percent(1) - DIFFUSION_SHARE_CAP_PERCENT).abs() < f32::EPSILON);
}
