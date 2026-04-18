//! Group A — `HasteConfig::multiplier` formula (pure unit tests).
//!
//! Uses `canonical_config()` (`base=20.0`, `per_level=10.0`) unless the
//! test specifies otherwise. Pins the design-doc stacking table values
//! (stacks 0, 1, 2, 3, 4, 5, 6, 10, 100).

use super::super::system::HasteConfig;

// ── Behavior 1 — stack 0 returns identity ────────────────────────────────

#[test]
fn multiplier_zero_stacks_is_one() {
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn multiplier_zero_stacks_is_one_with_zero_percents() {
    // Edge: both tuning knobs at zero still returns identity at stack 0.
    let cfg = HasteConfig {
        base_percent:      0.0,
        per_level_percent: 0.0,
    };
    assert!((cfg.multiplier(0) - 1.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — stack 1 returns base + 1 ────────────────────────────────

#[test]
fn multiplier_stack_one_is_base_plus_one() {
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(1) - 1.20).abs() < 1e-6);
}

#[test]
fn multiplier_stack_one_ignores_per_level_percent() {
    // Edge: at stack 1, `per_level_percent` contributes 0 (the "extra"
    // factor is `stacks - 1 = 0`). Different per-level values give the
    // same stack-1 result.
    let cfg = HasteConfig {
        base_percent:      50.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(1) - 1.50).abs() < 1e-6);
}

// ── Behavior 3 — stack 2 pins 1.30 ───────────────────────────────────────

#[test]
fn multiplier_stack_two_pins_design_doc_value() {
    // 1.0 + (20.0 + 10.0 * 1) / 100.0 = 1.30
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(2) - 1.30).abs() < 1e-6);
}

#[test]
fn multiplier_stack_two_only_per_level_knob_changes() {
    // Edge: changing `per_level_percent` from 10 to 5 changes stack 2's
    // value from 1.30 to 1.25. Proves per-level is the only knob that
    // moves stack 2's result.
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 5.0,
    };
    assert!((cfg.multiplier(2) - 1.25).abs() < 1e-6);
}

// ── Behavior 4 — stack 3 pins 1.40 ───────────────────────────────────────

#[test]
fn multiplier_stack_three_adds_two_levels() {
    // 1.0 + (20.0 + 10.0 * 2) / 100.0 = 1.40
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(3) - 1.40).abs() < 1e-6);
}

#[test]
fn multiplier_stack_three_with_zero_per_level_equals_stack_one() {
    // Edge: with `per_level_percent == 0.0`, all stacks ≥ 1 give the
    // same multiplier (1.0 + base/100).
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 0.0,
    };
    assert!((cfg.multiplier(3) - 1.20).abs() < 1e-6);
}

// ── Behavior 5 — stack 5 pins 1.60 ───────────────────────────────────────

#[test]
fn multiplier_stack_five_pins_design_doc_value() {
    // 1.0 + (20.0 + 10.0 * 4) / 100.0 = 1.60
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(5) - 1.60).abs() < 1e-6);
}

// ── Behavior 5A — bracketing stacks 4 and 6 ──────────────────────────────

#[test]
fn multiplier_stack_four_and_six_bracket_stack_five() {
    // Bracketing stacks 4 and 6 confirm the per-level step is constant.
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(4) - 1.50).abs() < 1e-6);
    assert!((cfg.multiplier(6) - 1.70).abs() < 1e-6);
}

// ── Behavior 6 — stack 10 pins 2.10 ──────────────────────────────────────

#[test]
fn multiplier_stack_ten_pins_design_doc_value() {
    // 1.0 + (20.0 + 10.0 * 9) / 100.0 = 2.10
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(10) - 2.10).abs() < 1e-6);
}

#[test]
fn multiplier_has_no_builtin_cap_at_large_stacks() {
    // Edge: stack 100 → 1.0 + (20.0 + 10.0 * 99) / 100.0 = 11.10.
    // No built-in cap — capping is the bolt domain's responsibility.
    let cfg = HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    };
    assert!((cfg.multiplier(100) - 11.10).abs() < 1e-4);
}

// ── Behavior 7 — per_level_percent == 0.0 is constant across stacks ─────

#[test]
fn multiplier_constant_when_per_level_is_zero() {
    let cfg = HasteConfig {
        base_percent:      25.0,
        per_level_percent: 0.0,
    };
    for k in [1u32, 2, 3, 5, 10] {
        let m = cfg.multiplier(k);
        assert!(
            (m - 1.25).abs() < 1e-6,
            "expected multiplier(1.25) at stack {k}, got {m}"
        );
    }
}

#[test]
fn multiplier_zero_stack_wins_over_constant_rate() {
    // Edge: even with a non-zero `base_percent`, stack 0 returns 1.0 —
    // the `stacks == 0` early return wins over the constant rate.
    let cfg = HasteConfig {
        base_percent:      25.0,
        per_level_percent: 0.0,
    };
    assert!((cfg.multiplier(0) - 1.0).abs() < f32::EPSILON);
}
