//! Group A — `OverchargeConfig::per_kill_multiplier` formula (pure unit tests).
//!
//! Uses `canonical_config()` (`base_frac=0.05`, `per_level_frac=0.03`) unless
//! the test specifies otherwise. Pins the design-doc stacking table values.

use super::{super::system::OverchargeConfig, helpers::canonical_config};

// ── Behavior 1 — stack 0 returns 1.0 (identity) ──────────────────────────

#[test]
fn per_kill_zero_stacks_is_one() {
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn per_kill_zero_stacks_is_one_with_zero_fracs() {
    // Edge: both tuning knobs at zero still returns identity at stack 0.
    let cfg = OverchargeConfig {
        base_frac:      0.0,
        per_level_frac: 0.0,
    };
    assert!((cfg.per_kill_multiplier(0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn per_kill_zero_stacks_short_circuits_large_fracs() {
    // Edge: even with large base/per-level fractions, stack 0 returns 1.0.
    // The `stacks == 0` early-return wins over any rate.
    let cfg = OverchargeConfig {
        base_frac:      0.5,
        per_level_frac: 0.5,
    };
    assert!((cfg.per_kill_multiplier(0) - 1.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — stack 1 returns 1.0 + base_frac ─────────────────────────

#[test]
fn per_kill_stack_one_is_base_plus_one() {
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(1) - 1.05).abs() < 1e-6);
}

#[test]
fn per_kill_stack_one_ignores_per_level_frac() {
    // Edge: at stack 1, per_level_frac contributes 0 because extra = 0.
    // A huge per_level_frac (0.99) does not affect stack 1's result.
    let cfg = OverchargeConfig {
        base_frac:      0.10,
        per_level_frac: 0.99,
    };
    assert!((cfg.per_kill_multiplier(1) - 1.10).abs() < 1e-6);
}

// ── Behavior 3 — stack 2 pins design-doc value 1.08 ──────────────────────

#[test]
fn per_kill_stack_two_pins_design_doc_value() {
    // 1.0 + 0.05 + 0.03 * 1 = 1.08
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(2) - 1.08).abs() < 1e-6);
}

#[test]
fn per_kill_stack_two_with_zero_per_level_equals_stack_one() {
    // Edge: per_level_frac = 0.0 makes stack 2 equal stack 1.
    let cfg = OverchargeConfig {
        base_frac:      0.05,
        per_level_frac: 0.0,
    };
    assert!((cfg.per_kill_multiplier(2) - 1.05).abs() < 1e-6);
}

// ── Behavior 4 — stack 3 pins design-doc value 1.11 ──────────────────────

#[test]
fn per_kill_stack_three_adds_two_levels() {
    // 1.0 + 0.05 + 0.03 * 2 = 1.11
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(3) - 1.11).abs() < 1e-6);
}

#[test]
fn per_kill_stack_three_with_zero_base_still_grows_per_level() {
    // Edge: base_frac == 0 still allows per-level growth.
    // 1.0 + 0.0 + 0.03 * 2 = 1.06
    let cfg = OverchargeConfig {
        base_frac:      0.0,
        per_level_frac: 0.03,
    };
    assert!((cfg.per_kill_multiplier(3) - 1.06).abs() < 1e-6);
}

// ── Behavior 5 — stack 5 extends the pattern linearly ────────────────────

#[test]
fn per_kill_stack_five_pins_linear_extension() {
    // 1.0 + 0.05 + 0.03 * 4 = 1.17
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(5) - 1.17).abs() < 1e-6);
}

#[test]
fn per_kill_stack_five_with_matching_per_level_frac() {
    // Edge: per_level_frac == base_frac == 0.05 — stack 5 is
    // 1.0 + 0.05 + 0.05 * 4 = 1.25. Pins linear scaling.
    let cfg = OverchargeConfig {
        base_frac:      0.05,
        per_level_frac: 0.05,
    };
    assert!((cfg.per_kill_multiplier(5) - 1.25).abs() < 1e-6);
}

// ── Behavior 6 — per_level_frac == 0.0 is constant across stacks ≥ 1 ────

#[test]
fn per_kill_constant_when_per_level_is_zero() {
    let cfg = OverchargeConfig {
        base_frac:      0.07,
        per_level_frac: 0.0,
    };
    for k in [1u32, 2, 3, 5, 10] {
        let m = cfg.per_kill_multiplier(k);
        assert!(
            (m - 1.07).abs() < 1e-6,
            "expected 1.07 at stack {k}, got {m}"
        );
    }
}

#[test]
fn per_kill_zero_stack_wins_over_constant_rate() {
    // Edge: stack 0 still returns 1.0 even with a non-zero base_frac.
    let cfg = OverchargeConfig {
        base_frac:      0.07,
        per_level_frac: 0.0,
    };
    assert!((cfg.per_kill_multiplier(0) - 1.0).abs() < f32::EPSILON);
}

// ── Behavior 7 — no built-in cap at large stack counts ───────────────────

#[test]
fn per_kill_stack_one_hundred_is_uncapped() {
    // 1.0 + 0.05 + 0.03 * 99 = 4.02 — no cap in the formula.
    let cfg = canonical_config();
    assert!((cfg.per_kill_multiplier(100) - 4.02).abs() < 1e-4);
}

#[test]
fn per_kill_stack_one_thousand_is_finite_and_large() {
    // Edge: extreme stack count is finite (no NaN/inf) and exceeds 30.
    let cfg = canonical_config();
    let result = cfg.per_kill_multiplier(1000);
    assert!(result.is_finite(), "multiplier must be finite");
    assert!(
        result > 30.0,
        "extreme stack count must grow past 30, got {result}"
    );
}
