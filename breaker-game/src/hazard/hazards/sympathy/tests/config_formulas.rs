//! Group A — `SympathyConfig` formula methods (Behaviors 1–17).
//!
//! Pure unit tests — no `App`, no ECS. Direct absolute-difference comparisons
//! for `heal_percent`; exact integer equality for `cascade_depth`.

use super::super::system::SympathyConfig;

// Canonical design-doc config used across most behaviours.
const fn canonical() -> SympathyConfig {
    SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  5.0,
        depth_increase_interval: 5,
    }
}

// ════════════════════════════════════════════════════════════════════════════
// heal_percent(stacks) pins — Behaviors 1–10
// ════════════════════════════════════════════════════════════════════════════

// ── Behavior 1 — heal_percent(0) short-circuits to 0.0 ──────────────────────

#[test]
fn heal_percent_zero_stacks_returns_zero() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(0) - 0.0).abs() < f32::EPSILON,
        "heal_percent(0) must short-circuit to 0.0, got {}",
        cfg.heal_percent(0)
    );
}

#[test]
fn heal_percent_zero_stacks_ignores_massive_per_level() {
    // Short-circuit MUST run before any arithmetic — pathological per-level
    // must NOT leak in.
    let cfg = SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  9999.0,
        depth_increase_interval: 5,
    };
    assert!(
        (cfg.heal_percent(0) - 0.0).abs() < f32::EPSILON,
        "heal_percent(0) must be 0.0 even with huge per_level, got {}",
        cfg.heal_percent(0)
    );
}

// ── Behavior 2 — heal_percent(1) returns base_heal_percent ──────────────────

#[test]
fn heal_percent_stack_one_returns_base() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(1) - 25.0).abs() < f32::EPSILON,
        "heal_percent(1) must equal base_heal_percent (25.0), got {}",
        cfg.heal_percent(1)
    );
}

#[test]
fn heal_percent_stack_one_ignores_large_per_level() {
    // Per-level must contribute nothing at stack 1 (extra == 0).
    let cfg = SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  999.0,
        depth_increase_interval: 5,
    };
    assert!(
        (cfg.heal_percent(1) - 25.0).abs() < f32::EPSILON,
        "per_level must NOT contribute at stack 1; got {}",
        cfg.heal_percent(1)
    );
}

// ── Behavior 3 — heal_percent(2) returns 30.0 ───────────────────────────────

#[test]
fn heal_percent_stack_two_returns_thirty() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(2) - 30.0).abs() < f32::EPSILON,
        "heal_percent(2) must equal 30.0, got {}",
        cfg.heal_percent(2)
    );
}

#[test]
fn heal_percent_stack_two_with_zero_base_returns_per_level() {
    // Per-level math runs even when base == 0.
    let cfg = SympathyConfig {
        base_heal_percent:       0.0,
        heal_per_level_percent:  30.0,
        depth_increase_interval: 5,
    };
    assert!(
        (cfg.heal_percent(2) - 30.0).abs() < f32::EPSILON,
        "zero-base + 30.0 per_level at stack 2 must return 30.0, got {}",
        cfg.heal_percent(2)
    );
}

// ── Behavior 4 — heal_percent(3) returns 35.0 ───────────────────────────────

#[test]
fn heal_percent_stack_three_returns_thirty_five() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(3) - 35.0).abs() < f32::EPSILON,
        "heal_percent(3) must equal 35.0 (design doc), got {}",
        cfg.heal_percent(3)
    );
}

// ── Behavior 5 — heal_percent(5) returns 45.0 ───────────────────────────────

#[test]
fn heal_percent_stack_five_returns_forty_five() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(5) - 45.0).abs() < f32::EPSILON,
        "heal_percent(5) must equal 45.0 (design doc), got {}",
        cfg.heal_percent(5)
    );
}

// ── Behavior 6 — heal_percent(6) returns 50.0 ───────────────────────────────

#[test]
fn heal_percent_stack_six_returns_fifty() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(6) - 50.0).abs() < f32::EPSILON,
        "heal_percent(6) must equal 50.0 (design doc; depth-boundary), got {}",
        cfg.heal_percent(6)
    );
}

// ── Behavior 7 — heal_percent(11) returns 75.0 ──────────────────────────────

#[test]
fn heal_percent_stack_eleven_returns_seventy_five() {
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(11) - 75.0).abs() < f32::EPSILON,
        "heal_percent(11) must equal 75.0 (design doc; depth-3 boundary), got {}",
        cfg.heal_percent(11)
    );
}

// ── Behavior 8 — heal_percent is NOT capped ─────────────────────────────────

#[test]
fn heal_percent_is_not_capped_at_twenty_stacks() {
    // 25 + 19 * 5 = 120. Design doc is silent on a cap; we intentionally
    // allow overheal at high stack counts.
    let cfg = canonical();
    assert!(
        (cfg.heal_percent(20) - 120.0).abs() < f32::EPSILON,
        "heal_percent(20) must equal 120.0 (uncapped); got {}",
        cfg.heal_percent(20)
    );
}

// ── Behavior 9 — heal_percent does not panic at u32::MAX ────────────────────

#[test]
fn heal_percent_does_not_panic_at_u32_max() {
    let cfg = canonical();
    let _value = cfg.heal_percent(u32::MAX);
    // Exact value not asserted — may saturate toward f32::INFINITY. The
    // spec-level guarantee is no-panic.
}

// ── Behavior 10 — heal_per_level == 0 yields constant base ──────────────────

#[test]
fn heal_percent_with_zero_per_level_is_constant_base() {
    let cfg = SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  0.0,
        depth_increase_interval: 5,
    };
    for k in [1u32, 2, 3, 10] {
        let v = cfg.heal_percent(k);
        assert!(
            (v - 25.0).abs() < f32::EPSILON,
            "per_level=0 must return base=25.0 at stack {k}; got {v}"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// cascade_depth(stacks) pins — Behaviors 11–17
// ════════════════════════════════════════════════════════════════════════════

// ── Behavior 11 — cascade_depth(0) short-circuits to 0 ──────────────────────

#[test]
fn cascade_depth_zero_stacks_returns_zero() {
    let cfg = canonical();
    assert_eq!(cfg.cascade_depth(0), 0);
}

#[test]
fn cascade_depth_zero_stacks_is_zero_even_with_zero_interval() {
    // Pathological depth_increase_interval=0 must still yield 0 at stacks=0.
    let cfg = SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  5.0,
        depth_increase_interval: 0,
    };
    assert_eq!(cfg.cascade_depth(0), 0);
}

// ── Behavior 12 — cascade_depth(1..=5) == 1 ─────────────────────────────────

#[test]
fn cascade_depth_one_through_five_is_one() {
    let cfg = canonical();
    for k in 1u32..=5 {
        assert_eq!(
            cfg.cascade_depth(k),
            1,
            "cascade_depth({k}) must equal 1 (design doc)"
        );
    }
}

// ── Behavior 13 — cascade_depth(6..=10) == 2 ────────────────────────────────

#[test]
fn cascade_depth_six_through_ten_is_two() {
    let cfg = canonical();
    for k in 6u32..=10 {
        assert_eq!(
            cfg.cascade_depth(k),
            2,
            "cascade_depth({k}) must equal 2 (design doc)"
        );
    }
}

// ── Behavior 14 — cascade_depth(11..=15) == 3 ───────────────────────────────

#[test]
fn cascade_depth_eleven_through_fifteen_is_three() {
    let cfg = canonical();
    for k in 11u32..=15 {
        assert_eq!(
            cfg.cascade_depth(k),
            3,
            "cascade_depth({k}) must equal 3 (design doc)"
        );
    }
}

// ── Behavior 15 — cascade_depth(50) == 10 ───────────────────────────────────

#[test]
fn cascade_depth_stack_fifty_is_ten() {
    let cfg = canonical();
    // 1 + (50 - 1) / 5 = 1 + 9 = 10.
    assert_eq!(cfg.cascade_depth(50), 10);
}

// ── Behavior 16 — pathological depth_increase_interval == 0 no-panic ────────

#[test]
fn cascade_depth_with_zero_interval_is_clamped_to_one() {
    let cfg = SympathyConfig {
        base_heal_percent:       25.0,
        heal_per_level_percent:  5.0,
        depth_increase_interval: 0,
    };
    // The interval is clamped to 1 internally; depth pins at 1 for stacks >= 1.
    assert_eq!(cfg.cascade_depth(10), 1);
}

// ── Behavior 17 — cascade_depth does not panic at u32::MAX ──────────────────

#[test]
fn cascade_depth_does_not_panic_at_u32_max() {
    let cfg = canonical();
    let _value = cfg.cascade_depth(u32::MAX);
}
