//! Group A — `DriftConfig::force_magnitude` formula (pure unit tests).
//!
//! Uses `canonical_config()` (`force=100.0, period_secs=8.0,
//! per_level_force=33.3`) unless the test specifies otherwise. Pins the
//! shipped formula: `0.0` at stack 0, else `force + per_level_force * (stacks - 1)`.

use super::{super::system::DriftConfig, helpers::canonical_config};

// ── Behavior 1 — stack 0 returns 0.0 (identity) ──────────────────────────

#[test]
fn force_zero_stacks_is_zero() {
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    };
    assert!((cfg.force_magnitude(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn force_zero_stacks_is_zero_even_with_nonzero_force() {
    // Edge: `force > 0` but `stacks == 0` still yields 0.0 — the
    // early-return short-circuits any nonzero rate.
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    };
    assert!((cfg.force_magnitude(0) - 0.0).abs() < f32::EPSILON);
    // Bitwise-preservation: the returned value is literal 0.0_f32.
    assert_eq!(cfg.force_magnitude(0).to_bits(), 0.0_f32.to_bits());
}

// ── Behavior 2 — stack 1 returns `force` (base only) ─────────────────────

#[test]
fn force_stack_one_equals_base() {
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    };
    assert!((cfg.force_magnitude(1) - 100.0).abs() < 1e-4);
}

#[test]
fn force_stack_one_ignores_per_level_force() {
    // Edge: large `per_level_force` is irrelevant at stack 1 (extra=0).
    let cfg = DriftConfig {
        force:           250.0,
        period_secs:     8.0,
        per_level_force: 999.0,
    };
    assert!((cfg.force_magnitude(1) - 250.0).abs() < 1e-4);
}

// ── Behavior 3 — stack 2 pins `force + per_level_force` = 133.3 ──────────

#[test]
fn force_stack_two_adds_one_level() {
    let cfg = canonical_config();
    assert!((cfg.force_magnitude(2) - 133.3).abs() < 1e-4);
}

#[test]
fn force_stack_two_with_zero_per_level_equals_stack_one() {
    // Edge: `per_level_force == 0` → stack 2 equals stack 1.
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 0.0,
    };
    assert!((cfg.force_magnitude(2) - 100.0).abs() < 1e-4);
}

// ── Behavior 4 — stack 3 pins `force + 2 * per_level_force` = 166.6 ──────

#[test]
fn force_stack_three_adds_two_levels() {
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    };
    // 100 + 33.3 * 2 = 166.6
    assert!((cfg.force_magnitude(3) - 166.6).abs() < 1e-4);
}

#[test]
fn force_stack_three_with_zero_base_still_grows_per_level() {
    // Edge: `force == 0` does NOT silence per-level growth (0 + 50*2 = 100).
    let cfg = DriftConfig {
        force:           0.0,
        period_secs:     8.0,
        per_level_force: 50.0,
    };
    assert!((cfg.force_magnitude(3) - 100.0).abs() < 1e-4);
}

// ── Behavior 5 — stack 5 extends linearly ─────────────────────────────────

#[test]
fn force_stack_five_extends_pattern_linearly() {
    let cfg = canonical_config();
    // 100 + 33.3 * 4 = 233.2
    assert!((cfg.force_magnitude(5) - 233.2).abs() < 1e-4);
}

#[test]
fn force_stack_five_with_small_per_level_scales_proportionally() {
    // Edge: `per_level_force = 10` → 100 + 10*4 = 140.
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 10.0,
    };
    assert!((cfg.force_magnitude(5) - 140.0).abs() < 1e-4);
}

// ── Behavior 6 — stack 10 extends linearly ────────────────────────────────

#[test]
fn force_stack_ten_extends_pattern_linearly() {
    let cfg = canonical_config();
    // 100 + 33.3 * 9 = 399.7
    assert!((cfg.force_magnitude(10) - 399.7).abs() < 1e-3);
}

#[test]
fn force_stack_ten_with_zero_per_level_equals_base() {
    // Edge: per_level=0 is constant at stack 10.
    let cfg = DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 0.0,
    };
    assert!((cfg.force_magnitude(10) - 100.0).abs() < 1e-4);
}

// ── Behavior 7 — per_level_force == 0 is constant across stacks ≥ 1 ──────

#[test]
fn per_level_force_zero_is_constant_across_stacks_ge_1() {
    let cfg = DriftConfig {
        force:           75.0,
        period_secs:     8.0,
        per_level_force: 0.0,
    };
    for k in [1_u32, 2, 3, 5, 10] {
        assert!(
            (cfg.force_magnitude(k) - 75.0).abs() < 1e-4,
            "stack {k} must produce 75.0 with per_level_force=0"
        );
    }
}

#[test]
fn per_level_force_zero_still_returns_zero_at_stack_zero() {
    // Edge: stack 0 early-return wins over the constant rate.
    let cfg = DriftConfig {
        force:           75.0,
        period_secs:     8.0,
        per_level_force: 0.0,
    };
    assert!((cfg.force_magnitude(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 8 — force == 0 is zero at all stacks ─────────────────────────

#[test]
fn force_zero_and_per_level_zero_is_zero_at_all_stacks() {
    let cfg = DriftConfig {
        force:           0.0,
        period_secs:     8.0,
        per_level_force: 0.0,
    };
    for k in [0_u32, 1, 2, 3, 5, 10] {
        assert!(
            (cfg.force_magnitude(k) - 0.0).abs() < f32::EPSILON,
            "stack {k} must produce 0.0 with force=0 and per_level=0"
        );
    }
}

#[test]
fn force_zero_with_nonzero_per_level_allows_per_level_growth() {
    // Edge: base is additive, not multiplicative — force=0 still lets
    // per_level grow the effective magnitude.
    let cfg = DriftConfig {
        force:           0.0,
        period_secs:     8.0,
        per_level_force: 50.0,
    };
    assert!((cfg.force_magnitude(3) - 100.0).abs() < 1e-4);
}

// ── Behavior 9 — no built-in cap at large stacks ──────────────────────────

#[test]
fn force_magnitude_has_no_cap_at_large_stacks() {
    let cfg = canonical_config();
    // 100 + 33.3 * 99 = 3396.7
    assert!((cfg.force_magnitude(100) - 3396.7).abs() < 1e-3);
}

#[test]
fn force_magnitude_at_thousand_stacks_is_finite_and_large() {
    // Edge: 1000 stacks — finite and > 30000.
    let cfg = canonical_config();
    let result = cfg.force_magnitude(1000);
    assert!(result.is_finite(), "force_magnitude(1000) must be finite");
    assert!(
        result > 30_000.0,
        "force_magnitude(1000) must exceed 30000, got {result}"
    );
}

// ── Behavior 10 — u32::MAX stacks yields finite (defensive) ──────────────

#[test]
fn force_magnitude_u32_max_is_finite_and_positive() {
    let cfg = canonical_config();
    let result = cfg.force_magnitude(u32::MAX);
    assert!(
        result.is_finite(),
        "force_magnitude(u32::MAX) must be finite, got {result}"
    );
    assert!(
        result > 0.0,
        "force_magnitude(u32::MAX) must be positive, got {result}"
    );
}

#[test]
fn force_magnitude_pathological_inputs_never_produce_nan() {
    // Edge: pathological tuning with enormous base + per_level at u32::MAX
    // may saturate to infinity but MUST NOT produce NaN.
    let cfg = DriftConfig {
        force:           f32::MAX / 2.0,
        period_secs:     8.0,
        per_level_force: f32::MAX / 2.0,
    };
    let result = cfg.force_magnitude(u32::MAX);
    assert!(
        !result.is_nan(),
        "pathological inputs must not produce NaN, got {result}"
    );
}
