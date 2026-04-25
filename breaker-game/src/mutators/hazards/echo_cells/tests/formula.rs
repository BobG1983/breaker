use super::{super::system::*, helpers::*};

// ── ghost_hp formula ──────────────────────────────────────────────────

#[test]
fn ghost_hp_zero_stacks_is_zero() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    };
    assert!((cfg.ghost_hp(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn ghost_hp_stack_one_is_base() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    };
    assert!((cfg.ghost_hp(1) - 1.0).abs() < 1e-5);
}

#[test]
fn ghost_hp_doubles_per_stack() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    };
    assert!((cfg.ghost_hp(3) - 4.0).abs() < 1e-5);
    assert!((cfg.ghost_hp(5) - 16.0).abs() < 1e-4);
}

// ── A. ghost_hp formula — extended coverage ───────────────────────────

// Behavior 1 — stack 2 is exactly base * multiplier.
#[test]
fn ghost_hp_stack_two_is_exactly_base_times_multiplier() {
    let cfg = canonical_config();
    assert!((cfg.ghost_hp(2) - 2.0).abs() < 1e-5);
}

#[test]
fn ghost_hp_stack_two_with_base_five_multiplier_two_is_ten() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              5.0,
        per_level_multiplier: 2.0,
    };
    assert!((cfg.ghost_hp(2) - 10.0).abs() < 1e-5);
}

// Behavior 2 — non-doubling multiplier scales geometrically.
#[test]
fn ghost_hp_with_non_doubling_multiplier_scales_correctly() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              2.0,
        per_level_multiplier: 1.5,
    };
    // 2.0 * 1.5^2 = 4.5
    assert!((cfg.ghost_hp(3) - 4.5).abs() < 1e-4);
}

#[test]
fn ghost_hp_with_identity_multiplier_is_constant_across_stacks() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              3.0,
        per_level_multiplier: 1.0,
    };
    for k in [1u32, 2, 3, 5, 10] {
        assert!((cfg.ghost_hp(k) - 3.0).abs() < 1e-5, "k={k}");
    }
}

// Behavior 3 — sub-one multiplier decays.
#[test]
fn ghost_hp_with_sub_one_multiplier_decays() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              4.0,
        per_level_multiplier: 0.5,
    };
    // 4.0 * 0.5^2 = 1.0
    assert!((cfg.ghost_hp(3) - 1.0).abs() < 1e-5);
}

#[test]
fn ghost_hp_with_zero_multiplier_at_stack_two_is_zero() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              4.0,
        per_level_multiplier: 0.0,
    };
    // 4.0 * 0.0^1 = 0.0
    assert!(cfg.ghost_hp(2).abs() < f32::EPSILON);
}

// Behavior 4 — zero base HP is always zero.
#[test]
fn ghost_hp_zero_base_is_always_zero() {
    let cfg = EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              0.0,
        per_level_multiplier: 2.0,
    };
    for k in [0u32, 1, 2, 3, 5, 10] {
        assert!(cfg.ghost_hp(k).abs() < f32::EPSILON, "k={k}");
    }
}

// Behavior 5 — u32::MAX wraps via .cast_signed() to a finite value.
#[test]
fn ghost_hp_u32_max_wraps_via_cast_signed() {
    // stacks.saturating_sub(1) = u32::MAX-1 = 4294967294u32;
    // .cast_signed() bit-reinterprets to -2i32; 2.0.powi(-2) = 0.25.
    let cfg = canonical_config();
    let r = cfg.ghost_hp(u32::MAX);
    assert!(r.is_finite());
    assert!((r - 0.25).abs() < 1e-4);
}

// Behavior 5a — overflow pin: beyond the f32 threshold, powi saturates to +INF.
#[test]
fn ghost_hp_overflows_to_infinity_beyond_f32_max() {
    let cfg = canonical_config();
    // 2.0^128 ≈ 3.4e38 is above f32::MAX → powi saturates to +INF.
    let r = cfg.ghost_hp(129);
    assert!(r.is_infinite());
    assert!(r > 0.0);
}

// Behavior 5b — paired finite bracket: one stack below the threshold stays in-range.
#[test]
fn ghost_hp_at_overflow_threshold_minus_one_is_finite() {
    let cfg = canonical_config();
    // 2.0^127 ≈ 1.7e38 is within f32::MAX.
    let r = cfg.ghost_hp(128);
    assert!(r.is_finite());
    assert!(r > 0.0);
}

// Behavior 6 — Copy semantics allow repeated calls.
#[test]
fn ghost_hp_copy_semantics_allow_repeated_call() {
    let cfg = canonical_config();
    let a = cfg.ghost_hp(3);
    let b = cfg.ghost_hp(3);
    assert!((a - 4.0).abs() < 1e-5);
    assert!((b - 4.0).abs() < 1e-5);
    // cfg still usable afterwards — pins #[derive(Clone, Copy)].
    assert!((cfg.ghost_hp(1) - 1.0).abs() < 1e-5);
}
