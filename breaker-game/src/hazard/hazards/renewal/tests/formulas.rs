//! Group A — `RenewalConfig::duration_secs` formula (pure unit tests).
//!
//! No app, no systems. These validate the multiplicative diminishing-returns
//! formula `duration = base * (1 - frac)^(stacks - 1)`, with `duration_secs(0)
//! == 0.0`.

use super::super::system::RenewalConfig;

// ── Behavior 1 — Stack 0 yields zero duration ────────────────────────────

#[test]
fn duration_zero_stacks_is_zero() {
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn duration_zero_stacks_is_zero_even_with_huge_base_and_zero_frac() {
    // Edge: huge base + zero frac — still 0.0 at stack 0.
    let cfg = RenewalConfig {
        base_period_secs:         999.0,
        per_level_reduction_frac: 0.0,
    };
    assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — Stack 1 returns base period exactly ─────────────────────

#[test]
fn duration_stack_one_equals_base() {
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    assert!((cfg.duration_secs(1) - 10.0).abs() < 1e-5);
}

#[test]
fn duration_stack_one_equals_base_with_zero_frac() {
    // Edge: `per_level_reduction_frac = 0.0` — stack 1 is still base.
    let cfg = RenewalConfig {
        base_period_secs:         5.0,
        per_level_reduction_frac: 0.0,
    };
    assert!((cfg.duration_secs(1) - 5.0).abs() < 1e-5);
}

// ── Behavior 3 — Stack 3 applies diminishing returns ─────────────────────

#[test]
fn duration_stack_three_uses_diminishing_returns() {
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    // 10.0 * 0.8^2 = 6.4
    assert!((cfg.duration_secs(3) - 6.4).abs() < 1e-4);
}

#[test]
fn duration_stack_five_applies_four_reductions() {
    // Edge: stack 5 = 10.0 * 0.8^4 = 4.096
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    assert!((cfg.duration_secs(5) - 4.096).abs() < 1e-3);
}

// ── Behavior 4 — Stack 10 matches design-doc pinned value ≈ 1.342 ─────────

#[test]
fn duration_stack_ten_is_design_doc_pinned_value() {
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    // 10.0 * 0.8^9 ≈ 1.342
    let val = cfg.duration_secs(10);
    assert!(
        (val - 1.342).abs() < 1e-3,
        "stack 10 should be ≈ 1.342, got {val}"
    );
}

#[test]
fn duration_stack_twenty_is_positive_and_tiny_but_nonzero() {
    // Edge: at stack 20, duration is vanishingly small but never 0 (floor).
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 0.2,
    };
    let val = cfg.duration_secs(20);
    assert!(val > 0.0, "stack 20 must remain positive, got {val}");
    assert!(val < 0.15, "stack 20 should be < 0.15, got {val}");
}

// ── Behavior 5 — frac == 1.0 collapses to zero beyond stack 1 ────────────

#[test]
fn duration_frac_one_at_stack_two_is_zero() {
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 1.0,
    };
    // factor = (1.0 - 1.0) = 0.0; 10 * 0.0^1 = 0.0
    assert!((cfg.duration_secs(2) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn duration_frac_one_at_stack_one_is_base() {
    // Edge: `factor^0 = 1.0`, so stack 1 still equals base even with frac=1.0.
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 1.0,
    };
    assert!((cfg.duration_secs(1) - 10.0).abs() < 1e-5);
}

// ── Behavior 6 — frac > 1.0 is clamped to 0 factor ───────────────────────

#[test]
fn duration_frac_above_one_is_clamped_to_zero_factor() {
    // 1.0 - 1.5 = -0.5, clamped to 0.0 by `.max(0.0)`. 10 * 0.0^1 = 0.0.
    // Guards against a naive `powi` producing negative durations.
    let cfg = RenewalConfig {
        base_period_secs:         10.0,
        per_level_reduction_frac: 1.5,
    };
    assert!((cfg.duration_secs(2) - 0.0).abs() < f32::EPSILON);
}
