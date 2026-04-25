//! Group A — `GreedConfig` / `GreedStacks` pure-formula behaviors
//! (Behaviors 1–6).
//!
//! All tests call `GreedStacks::rarity_boost` directly; no `App`.

use super::super::system::{GreedConfig, GreedStacks};

// ── Behavior 1 — default() starts at zero skips ─────────────────────────────

#[test]
fn greed_stacks_default_starts_at_zero_skips() {
    let stacks = GreedStacks::default();
    assert_eq!(stacks.skips, 0, "GreedStacks::default().skips must be 0");
}

#[test]
fn greed_stacks_default_rarity_boost_is_zero_for_five_per_skip() {
    let stacks = GreedStacks::default();
    let cfg = GreedConfig {
        rarity_boost_per_skip: 5.0,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 0.0).abs() < f32::EPSILON,
        "default GreedStacks should return 0.0 boost with per-skip 5.0"
    );
}

// ── Behavior 2 — rarity_boost == 0.0 when skips == 0 regardless of per-skip ──

#[test]
fn rarity_boost_zero_skips_with_five_per_skip_is_zero() {
    let stacks = GreedStacks { skips: 0 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 5.0,
    };
    assert!((stacks.rarity_boost(cfg) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn rarity_boost_zero_skips_with_one_hundred_per_skip_is_zero() {
    // Edge case: large per-skip value must not leak when skips is zero.
    let stacks = GreedStacks { skips: 0 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 100.0,
    };
    assert!((stacks.rarity_boost(cfg) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 3 — one skip × per-skip value ──────────────────────────────────

#[test]
fn rarity_boost_one_skip_equals_per_skip_value() {
    let stacks = GreedStacks { skips: 1 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 5.0,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 5.0).abs() < f32::EPSILON,
        "1 skip × 5.0/skip should be 5.0, got {}",
        stacks.rarity_boost(cfg)
    );
}

// ── Behavior 4 — linear accumulation across multiple skips ──────────────────

#[test]
fn rarity_boost_three_skips_is_fifteen() {
    let stacks = GreedStacks { skips: 3 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 5.0,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 15.0).abs() < f32::EPSILON,
        "3 × 5.0 should be 15.0, got {}",
        stacks.rarity_boost(cfg)
    );
}

#[test]
fn rarity_boost_ten_skips_is_fifty() {
    // Edge case: larger stack count, linear scaling.
    let stacks = GreedStacks { skips: 10 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 5.0,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 50.0).abs() < f32::EPSILON,
        "10 × 5.0 should be 50.0, got {}",
        stacks.rarity_boost(cfg)
    );
}

// ── Behavior 5 — non-trivial fractional per-skip values ─────────────────────

#[test]
fn rarity_boost_four_skips_with_seven_point_five_per_skip_is_thirty() {
    let stacks = GreedStacks { skips: 4 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 7.5,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 30.0).abs() < 1e-4,
        "4 × 7.5 should be 30.0, got {}",
        stacks.rarity_boost(cfg)
    );
}

// ── Behavior 6 — zero per-skip collapses to zero boost ──────────────────────

#[test]
fn rarity_boost_with_zero_per_skip_is_always_zero() {
    let stacks = GreedStacks { skips: 100 };
    let cfg = GreedConfig {
        rarity_boost_per_skip: 0.0,
    };
    assert!(
        (stacks.rarity_boost(cfg) - 0.0).abs() < f32::EPSILON,
        "100 × 0.0 should be 0.0, got {}",
        stacks.rarity_boost(cfg)
    );
}
