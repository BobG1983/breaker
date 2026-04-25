//! Groups A + B — `GravitySurgeConfig::duration_secs` + `::strength`
//! formula pins. Pure unit tests — no `App`, no systems.

use super::{super::system::GravitySurgeConfig, helpers::canonical_config};

// ── Group A — duration_secs formula ──────────────────────────────────────

// ── Behavior 1 — stack 0 returns 0.0 (identity) — PRESERVED ──────────────

#[test]
fn duration_zero_stacks_is_zero() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn duration_zero_stacks_nonzero_base_still_zero() {
    // Edge: stacks=0 early-return short-circuits any nonzero base.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      100.0,
        per_level_duration_secs: 50.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — stack 1 returns base_duration_secs (base only) ──────────

#[test]
fn duration_stack_one_returns_base() {
    let cfg = canonical_config();
    assert!((cfg.duration_secs(1) - 2.0).abs() < 1e-6);
}

#[test]
fn duration_stack_one_ignores_per_level() {
    // Edge: stack 1 ignores per_level_duration_secs since (stacks-1)==0.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 999.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(1) - 2.0).abs() < 1e-6);
}

// ── Behavior 3 — stack 3 returns base + per_level * 2 = 4.0 — PRESERVED ──

#[test]
fn duration_scales_linearly_with_stacks() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(1) - 2.0).abs() < 1e-6);
    assert!((cfg.duration_secs(3) - 4.0).abs() < 1e-6);
}

#[test]
fn duration_zero_base_still_allows_per_level_growth() {
    // Edge: base=0 with per_level=2 at stack 3 → 0 + 2*2 = 4.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      0.0,
        per_level_duration_secs: 2.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(3) - 4.0).abs() < 1e-6);
}

// ── Behavior 4 — stack 5 extends the pattern linearly ────────────────────

#[test]
fn duration_stack_five_extends_linearly() {
    let cfg = canonical_config();
    assert!((cfg.duration_secs(5) - 6.0).abs() < 1e-6);
}

#[test]
fn duration_stack_five_with_half_rate() {
    // Edge: per_level=0.5 at stack 5 → 2 + 0.5*4 = 4.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 0.5,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(5) - 4.0).abs() < 1e-5);
}

// ── Behavior 5 — stack 10 extends the pattern linearly ───────────────────

#[test]
fn duration_stack_ten_extends_linearly() {
    let cfg = canonical_config();
    assert!((cfg.duration_secs(10) - 11.0).abs() < 1e-5);
}

#[test]
fn duration_stack_ten_constant_rate_when_per_level_zero() {
    // Edge: per_level=0 at stack 10 → returns base=2.0.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 0.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(10) - 2.0).abs() < 1e-6);
}

// ── Behavior 6 — per_level_duration_secs == 0 constant across stacks ────

#[test]
fn duration_constant_when_per_level_zero() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      3.5,
        per_level_duration_secs: 0.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    for k in [1u32, 2, 3, 5, 10] {
        assert!(
            (cfg.duration_secs(k) - 3.5).abs() < 1e-5,
            "stack {k} should return 3.5, got {}",
            cfg.duration_secs(k)
        );
    }
}

#[test]
fn duration_zero_early_return_wins_over_constant_rate() {
    // Edge: stacks=0 still returns 0 even with constant-rate config.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      3.5,
        per_level_duration_secs: 0.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 7 — base_duration_secs == 0.0 zero at all stacks when ──────
// per_level == 0.0

#[test]
fn duration_all_zero_stays_zero_for_all_stacks() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      0.0,
        per_level_duration_secs: 0.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    for k in [0u32, 1, 2, 3, 5, 10] {
        assert!(
            (cfg.duration_secs(k) - 0.0).abs() < f32::EPSILON,
            "stack {k} should return 0.0, got {}",
            cfg.duration_secs(k)
        );
    }
}

#[test]
fn duration_zero_base_with_positive_per_level_grows() {
    // Edge: base=0, per_level=0.5 at stack 3 → 0 + 0.5*2 = 1.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      0.0,
        per_level_duration_secs: 0.5,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.duration_secs(3) - 1.0).abs() < 1e-6);
}

// ── Behavior 8 — u32::MAX stacks yields a finite value ──────────────────

#[test]
fn duration_at_u32_max_is_finite_and_positive() {
    let cfg = canonical_config();
    let result = cfg.duration_secs(u32::MAX);
    assert!(result.is_finite(), "result should be finite, got {result}");
    assert!(result > 0.0, "result should be positive, got {result}");
}

#[test]
fn duration_at_u32_max_with_pathological_rate_is_not_nan() {
    // Edge: per_level near f32::MAX/2 at u32::MAX — may produce Inf but
    // NEVER NaN.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      0.0,
        per_level_duration_secs: f32::MAX / 2.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    let result = cfg.duration_secs(u32::MAX);
    assert!(!result.is_nan(), "result must not be NaN, got {result}");
}

// ── Group B — strength formula ──────────────────────────────────────────

// ── Behavior 9 — stack 0 returns 0.0 (identity) — PRESERVED ─────────────

#[test]
fn strength_zero_stacks_is_zero() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    assert!((cfg.strength(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn strength_zero_stacks_nonzero_base_still_zero() {
    // Edge: stacks=0 early-return wins over nonzero base+frac.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           9999.0,
        per_level_strength_frac: 100.0,
    };
    assert!((cfg.strength(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 10 — stack 1 returns base_strength (sqrt(0)=0) — PRESERVED ─

#[test]
fn strength_stack_one_returns_base() {
    let cfg = canonical_config();
    assert!((cfg.strength(1) - 500.0).abs() < 1e-4);
}

#[test]
fn strength_stack_one_ignores_frac() {
    // Edge: stack 1 ignores per_level_strength_frac since sqrt(0) == 0.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 9999.0,
    };
    assert!((cfg.strength(1) - 500.0).abs() < 1e-4);
}

// ── Behavior 11 — stack 5 pins base * (1 + 2*frac) = 1000 — PRESERVED ───

#[test]
fn strength_has_sqrt_diminishing_returns() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    };
    // Stack 1: 500 * (1 + 0.5 * sqrt(0)) = 500
    assert!((cfg.strength(1) - 500.0).abs() < 1e-4);
    // Stack 5: 500 * (1 + 0.5 * sqrt(4)) = 500 * 2 = 1000
    assert!((cfg.strength(5) - 1000.0).abs() < 1e-4);
}

#[test]
fn strength_stack_five_with_zero_frac_is_base_only() {
    // Edge: per_level_strength_frac=0 silences diminishing growth.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.0,
    };
    assert!((cfg.strength(5) - 500.0).abs() < 1e-4);
}

// ── Behavior 12 — stack 10 pins base * (1 + 3*frac) ≈ 1250 ──────────────

#[test]
fn strength_stack_ten_sqrt_nine_is_three() {
    let cfg = canonical_config();
    // 500 * (1 + 0.5 * sqrt(9)) = 500 * 2.5 = 1250
    assert!((cfg.strength(10) - 1250.0).abs() < 1e-3);
}

#[test]
fn strength_stack_ten_scaled_tuning() {
    // Edge: 100 * (1 + 1 * 3) = 400.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           100.0,
        per_level_strength_frac: 1.0,
    };
    assert!((cfg.strength(10) - 400.0).abs() < 1e-3);
}

// ── Behavior 13 — per_level_strength_frac == 0 constant across stacks ──

#[test]
fn strength_constant_when_frac_zero() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           750.0,
        per_level_strength_frac: 0.0,
    };
    for k in [1u32, 2, 3, 5, 10] {
        assert!(
            (cfg.strength(k) - 750.0).abs() < 1e-4,
            "stack {k} should return 750.0, got {}",
            cfg.strength(k)
        );
    }
}

#[test]
fn strength_zero_early_return_wins_over_constant_frac() {
    // Edge: stacks=0 returns 0 even with frac=0 config.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           750.0,
        per_level_strength_frac: 0.0,
    };
    assert!((cfg.strength(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 14 — base_strength == 0 zero at all stacks ────────────────

#[test]
fn strength_zero_base_is_zero_for_all_stacks() {
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           0.0,
        per_level_strength_frac: 0.5,
    };
    for k in [0u32, 1, 2, 3, 5, 10] {
        assert!(
            (cfg.strength(k) - 0.0).abs() < f32::EPSILON,
            "stack {k} should return 0.0, got {}",
            cfg.strength(k)
        );
    }
}

#[test]
fn strength_zero_base_silences_pathological_frac() {
    // Edge: base=0 at stack 100 with large frac → still 0.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           0.0,
        per_level_strength_frac: 9999.0,
    };
    assert!((cfg.strength(100) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 15 — u32::MAX stacks yields a finite, positive strength ──

#[test]
fn strength_at_u32_max_is_finite_and_large() {
    let cfg = canonical_config();
    let result = cfg.strength(u32::MAX);
    assert!(result.is_finite(), "result should be finite, got {result}");
    assert!(result > 500.0, "result should exceed base, got {result}");
    assert!(result > 1.0e7, "expected >1e7, got {result}");
}

#[test]
fn strength_at_u32_max_with_pathological_tuning_is_not_nan() {
    // Edge: pathological base and frac — may produce Inf but NEVER NaN.
    let cfg = GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           f32::MAX / 2.0,
        per_level_strength_frac: f32::MAX / 2.0,
    };
    let result = cfg.strength(u32::MAX);
    assert!(!result.is_nan(), "result must not be NaN, got {result}");
}
