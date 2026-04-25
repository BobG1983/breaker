//! Group A — `MomentumConfig::heal_per_hit` formula (Behaviors 1–6).
//!
//! Pure unit tests — no `App`, no ECS. Pin the stack-0 short-circuit, stack-1
//! base, stack-2 `base+per_level`, stack-3 design-doc value, large-stack
//! saturation, and zero-per-level constant behavior.

use super::super::system::MomentumConfig;

// ── Behavior 1 — stack 0 returns 0.0 ────────────────────────────────────────

#[test]
fn stack_zero_returns_zero() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    assert!((cfg.heal_per_hit(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn stack_zero_returns_zero_even_with_large_config() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      999.0,
        per_level_hp_per_hit: 999.0,
    };
    assert!((cfg.heal_per_hit(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — stack 1 returns base_hp_per_hit ────────────────────────────

#[test]
fn stack_one_returns_base_hp_per_hit() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    assert!(
        (cfg.heal_per_hit(1) - 10.0).abs() < f32::EPSILON,
        "stack-1 heal must equal base_hp_per_hit, got {}",
        cfg.heal_per_hit(1)
    );
}

#[test]
fn stack_one_ignores_large_per_level() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 999.0,
    };
    assert!(
        (cfg.heal_per_hit(1) - 10.0).abs() < f32::EPSILON,
        "per_level must NOT contribute at stack 1, got {}",
        cfg.heal_per_hit(1)
    );
}

// ── Behavior 3 — stack 2 returns base + per_level ───────────────────────────

#[test]
fn stack_two_returns_base_plus_per_level() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    assert!(
        (cfg.heal_per_hit(2) - 20.0).abs() < f32::EPSILON,
        "stack-2 heal must equal base + per_level (20.0), got {}",
        cfg.heal_per_hit(2)
    );
}

#[test]
fn stack_two_with_zero_base_uses_per_level() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      0.0,
        per_level_hp_per_hit: 5.0,
    };
    assert!(
        (cfg.heal_per_hit(2) - 5.0).abs() < f32::EPSILON,
        "zero base + 5.0 per_level at stack 2 must return 5.0, got {}",
        cfg.heal_per_hit(2)
    );
}

// ── Behavior 4 — stack 3 returns base + 2 * per_level ───────────────────────

#[test]
fn stack_three_returns_base_plus_two_per_level() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    // 10.0 + 10.0 * 2 = 30.0 (design-doc pinned)
    assert!(
        (cfg.heal_per_hit(3) - 30.0).abs() < f32::EPSILON,
        "stack-3 heal must equal design-doc 30.0, got {}",
        cfg.heal_per_hit(3)
    );
}

// ── Behavior 5 — stack 9 returns base + 8 * per_level ───────────────────────

#[test]
fn stack_nine_returns_base_plus_eight_per_level() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    // 10.0 + 10.0 * 8 = 90.0
    assert!(
        (cfg.heal_per_hit(9) - 90.0).abs() < f32::EPSILON,
        "stack-9 heal must equal 90.0, got {}",
        cfg.heal_per_hit(9)
    );
}

#[test]
fn stack_max_u32_does_not_panic() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 10.0,
    };
    // We don't care about the exact result — only that it doesn't panic
    // (saturating_sub pattern prevents underflow; f32 may saturate at infinity).
    let _value = cfg.heal_per_hit(u32::MAX);
}

// ── Behavior 6 — per_level == 0.0 yields constant base ──────────────────────

#[test]
fn per_level_zero_yields_constant_base() {
    let cfg = MomentumConfig {
        base_hp_per_hit:      10.0,
        per_level_hp_per_hit: 0.0,
    };
    for k in [1u32, 2, 3, 10] {
        let v = cfg.heal_per_hit(k);
        assert!(
            (v - 10.0).abs() < f32::EPSILON,
            "per_level=0 must return base=10.0 at stack {k}, got {v}"
        );
    }
}
