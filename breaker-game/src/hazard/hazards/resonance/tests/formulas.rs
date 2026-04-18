//! Group A — pure `ResonanceConfig` formula helpers.

use super::{super::system::ResonanceConfig, helpers::canonical_config};

// ── A1 — effective_window(0) == 0.0 ──────────────────────────────────────

#[test]
fn a1_effective_window_zero_stacks_is_zero() {
    let cfg = canonical_config();
    let got = cfg.effective_window(0);
    assert!(
        (got - 0.0).abs() < f32::EPSILON,
        "effective_window(0) should be 0.0, got {got}"
    );
}

// ── A2 — effective_window(1) == base_window ──────────────────────────────

#[test]
fn a2_effective_window_one_stack_equals_base_window() {
    let cfg = canonical_config();
    let got = cfg.effective_window(1);
    assert!(
        (got - 0.5).abs() < 1e-6,
        "effective_window(1) should be 0.5, got {got}"
    );
}

// ── A3 — linear scaling with stacks ─────────────────────────────────────

#[test]
fn a3_effective_window_three_stacks_equals_one_point_one() {
    let cfg = canonical_config();
    let got = cfg.effective_window(3);
    assert!(
        (got - 1.1).abs() < 1e-6,
        "effective_window(3) should be 1.1, got {got}"
    );
}

#[test]
fn a3_edge_effective_window_five_stacks_equals_one_point_seven() {
    let cfg = canonical_config();
    let got = cfg.effective_window(5);
    assert!(
        (got - 1.7).abs() < 1e-6,
        "effective_window(5) should be 1.7, got {got}"
    );
}

// ── A4 — effective_slow_duration(0) == 0.0 ───────────────────────────────

#[test]
fn a4_effective_slow_duration_zero_stacks_is_zero() {
    let cfg = canonical_config();
    let got = cfg.effective_slow_duration(0);
    assert!(
        (got - 0.0).abs() < f32::EPSILON,
        "effective_slow_duration(0) should be 0.0, got {got}"
    );
}

// ── A5 — effective_slow_duration(1) == base ─────────────────────────────

#[test]
fn a5_effective_slow_duration_one_stack_equals_base() {
    let cfg = canonical_config();
    let got = cfg.effective_slow_duration(1);
    assert!(
        (got - 1.5).abs() < 1e-5,
        "effective_slow_duration(1) should be 1.5 (ln(1)=0), got {got}"
    );
}

// ── A6 — effective_slow_duration(3) log scaling ────────────────────────

#[test]
fn a6_effective_slow_duration_three_stacks_applies_log_scaling() {
    let cfg = canonical_config();
    // Hand-computed: 1.5 * (1.0 + 0.2 * ln(3.0)) = 1.5 * 1.21972 ≈ 1.82958
    let got = cfg.effective_slow_duration(3);
    assert!(
        (got - 1.82958).abs() < 1e-4,
        "effective_slow_duration(3) should be ~1.82958, got {got}"
    );
}

#[test]
fn a6_edge_effective_slow_duration_five_stacks() {
    let cfg = canonical_config();
    // Hand-computed: 1.5 * (1.0 + 0.2 * ln(5.0)) = 1.5 * 1.32189 ≈ 1.98283
    let got = cfg.effective_slow_duration(5);
    assert!(
        (got - 1.98283).abs() < 1e-4,
        "effective_slow_duration(5) should be ~1.98283, got {got}"
    );
}

// ── A7 — effective_slow_strength(0) == 0.0 ─────────────────────────────

#[test]
fn a7_effective_slow_strength_zero_stacks_is_zero() {
    let cfg = canonical_config();
    let got = cfg.effective_slow_strength(0);
    assert!(
        (got - 0.0).abs() < f32::EPSILON,
        "effective_slow_strength(0) should be 0.0, got {got}"
    );
}

// ── A8 — effective_slow_strength(1) == base ─────────────────────────────

#[test]
fn a8_effective_slow_strength_one_stack_equals_base() {
    let cfg = canonical_config();
    let got = cfg.effective_slow_strength(1);
    assert!(
        (got - 0.5).abs() < 1e-5,
        "effective_slow_strength(1) should be 0.5, got {got}"
    );
}

// ── A9 — effective_slow_strength(3) log scaling ────────────────────────

#[test]
fn a9_effective_slow_strength_three_stacks_applies_log_scaling() {
    let cfg = canonical_config();
    // Hand-computed: 0.5 * (1.0 + 0.15 * ln(3.0)) = 0.5 * 1.16479 ≈ 0.58240
    let got = cfg.effective_slow_strength(3);
    assert!(
        (got - 0.58240).abs() < 1e-4,
        "effective_slow_strength(3) should be ~0.58240, got {got}"
    );
}

// ── A10 — slow_multiplier(0) == 1.0 ─────────────────────────────────────

#[test]
fn a10_slow_multiplier_zero_stacks_is_one() {
    let cfg = canonical_config();
    let got = cfg.slow_multiplier(0);
    assert!(
        (got - 1.0).abs() < f32::EPSILON,
        "slow_multiplier(0) should be 1.0 (no slow), got {got}"
    );
}

// ── A11 — slow_multiplier(1) == 1.0 - base_slow_strength ────────────────

#[test]
fn a11_slow_multiplier_one_stack_equals_one_minus_base_strength() {
    let cfg = canonical_config();
    let got = cfg.slow_multiplier(1);
    assert!(
        (got - 0.5).abs() < 1e-5,
        "slow_multiplier(1) should be 0.5 (1.0 - 0.5), got {got}"
    );
}

// ── A12 — slow_multiplier clamps at 0.1 ─────────────────────────────────

#[test]
fn a12_slow_multiplier_clamps_at_floor_when_strength_exceeds_one() {
    let cfg = ResonanceConfig {
        base_slow_strength: 0.99,
        slow_strength_scaling: 1.0,
        ..canonical_config()
    };
    // raw: 0.99 * (1 + 1*ln(10)) ≈ 3.27, multiplier raw = -2.27 → clamp 0.1.
    let got = cfg.slow_multiplier(10);
    assert!(
        (got - 0.1).abs() < 1e-5,
        "slow_multiplier clamp floor should fire at 0.1, got {got}"
    );
}

// ── A13 — slow_multiplier clamps at 1.0 ─────────────────────────────────

#[test]
fn a13_slow_multiplier_clamps_at_ceiling_when_strength_negative() {
    let cfg = ResonanceConfig {
        base_slow_strength: -0.25,
        ..canonical_config()
    };
    let got = cfg.slow_multiplier(1);
    assert!(
        (got - 1.0).abs() < 1e-5,
        "slow_multiplier should clamp to 1.0 ceiling, got {got}"
    );
}
