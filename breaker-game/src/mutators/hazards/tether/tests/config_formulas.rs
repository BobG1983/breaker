//! Section A — `TetherConfig` pure-formula behaviors (Behaviours 1–13).
//!
//! Each test pins a specific design-doc value for `damage_percent` and
//! `coverage_percent` at concrete stack counts, including the 100% coverage
//! cap.

use super::super::system::TetherConfig;

const fn canonical() -> TetherConfig {
    TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    }
}

// ── Behavior 1 — damage_percent(0) == 0.0 ────────────────────────────────────

#[test]
fn damage_percent_zero_stacks_is_zero() {
    let cfg = canonical();
    assert!((cfg.damage_percent(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn damage_percent_zero_stacks_with_large_base_is_zero() {
    let cfg = TetherConfig {
        base_damage:        999.0,
        damage_per_level:   10.0,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    };
    // stacks == 0 short-circuits BEFORE any base-value read.
    assert!((cfg.damage_percent(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 2 — damage_percent(1) == base_damage_percent ────────────────────

#[test]
fn damage_percent_stack_one_equals_base() {
    let cfg = canonical();
    assert!((cfg.damage_percent(1) - 25.0).abs() < f32::EPSILON);
}

#[test]
fn damage_percent_stack_one_ignores_per_level_multiplier() {
    let cfg = TetherConfig {
        base_damage:        25.0,
        damage_per_level:   999.0,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    };
    // per_level must not leak into stack 1 (pins (stacks - 1) factor == 0).
    assert!((cfg.damage_percent(1) - 25.0).abs() < f32::EPSILON);
}

// ── Behavior 3 — damage_percent(3) == 45.0 ───────────────────────────────────

#[test]
fn damage_percent_stack_three_is_forty_five() {
    let cfg = canonical();
    // 25 + 10 * 2 = 45.
    assert!((cfg.damage_percent(3) - 45.0).abs() < f32::EPSILON);
}

#[test]
fn damage_percent_stack_three_with_zero_base_still_grows() {
    let cfg = TetherConfig {
        base_damage:        0.0,
        damage_per_level:   22.5,
        base_coverage:      40.0,
        coverage_per_level: 10.0,
    };
    // 0 + 22.5 * 2 = 45.
    assert!((cfg.damage_percent(3) - 45.0).abs() < f32::EPSILON);
}

// ── Behavior 4 — damage_percent(5) == 65.0 ───────────────────────────────────

#[test]
fn damage_percent_stack_five_is_sixty_five() {
    let cfg = canonical();
    // 25 + 10 * 4 = 65.
    assert!((cfg.damage_percent(5) - 65.0).abs() < f32::EPSILON);
}

// ── Behavior 5 — damage_percent(7) == 85.0 ───────────────────────────────────

#[test]
fn damage_percent_stack_seven_is_eighty_five() {
    let cfg = canonical();
    // 25 + 10 * 6 = 85.
    assert!((cfg.damage_percent(7) - 85.0).abs() < f32::EPSILON);
}

// ── Behavior 6 — damage_percent(9) == 105.0 (uncapped) ───────────────────────

#[test]
fn damage_percent_stack_nine_is_one_hundred_five_uncapped() {
    let cfg = canonical();
    // 25 + 10 * 8 = 105.0 — design doc explicitly says uncapped.
    assert!((cfg.damage_percent(9) - 105.0).abs() < f32::EPSILON);
}

#[test]
fn damage_percent_is_uncapped_at_very_large_stacks() {
    let cfg = canonical();
    // 25 + 10 * 99 = 1015.0 — still uncapped.
    assert!((cfg.damage_percent(100) - 1015.0).abs() < f32::EPSILON);
}

// ── Behavior 7 — coverage_percent(0) == 0.0 ──────────────────────────────────

#[test]
fn coverage_percent_zero_stacks_is_zero() {
    let cfg = canonical();
    assert!((cfg.coverage_percent(0) - 0.0).abs() < f32::EPSILON);
}

#[test]
fn coverage_percent_zero_stacks_short_circuits_before_cap() {
    let cfg = TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      200.0,
        coverage_per_level: 10.0,
    };
    // Pins that stacks == 0 returns 0.0 BEFORE cap logic applies.
    assert!((cfg.coverage_percent(0) - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 8 — coverage_percent(1) == 40.0 ─────────────────────────────────

#[test]
fn coverage_percent_stack_one_equals_base() {
    let cfg = canonical();
    assert!((cfg.coverage_percent(1) - 40.0).abs() < f32::EPSILON);
}

#[test]
fn coverage_percent_stack_one_ignores_per_level_multiplier() {
    let cfg = TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      40.0,
        coverage_per_level: 999.0,
    };
    assert!((cfg.coverage_percent(1) - 40.0).abs() < f32::EPSILON);
}

// ── Behavior 9 — coverage_percent(3) == 60.0 ─────────────────────────────────

#[test]
fn coverage_percent_stack_three_is_sixty() {
    let cfg = canonical();
    // 40 + 10 * 2 = 60.
    assert!((cfg.coverage_percent(3) - 60.0).abs() < f32::EPSILON);
}

// ── Behavior 10 — coverage_percent(5) == 80.0 ────────────────────────────────

#[test]
fn coverage_percent_stack_five_is_eighty() {
    let cfg = canonical();
    // 40 + 10 * 4 = 80.
    assert!((cfg.coverage_percent(5) - 80.0).abs() < f32::EPSILON);
}

// ── Behavior 11 — coverage_percent(7) == 100.0 (inclusive cap boundary) ──────

#[test]
fn coverage_percent_stack_seven_is_exactly_one_hundred() {
    let cfg = canonical();
    // 40 + 10 * 6 = 100.0 — exactly at the cap; result equals cap.
    assert!((cfg.coverage_percent(7) - 100.0).abs() < f32::EPSILON);
}

// ── Behavior 12 — coverage_percent(9) is clamped to 100.0 ────────────────────

#[test]
fn coverage_percent_stack_nine_is_clamped_to_one_hundred() {
    let cfg = canonical();
    // Raw formula would be 40 + 10 * 8 = 120.0 → clamp to 100.0.
    assert!((cfg.coverage_percent(9) - 100.0).abs() < f32::EPSILON);
}

#[test]
fn coverage_percent_cap_engages_at_very_large_stacks() {
    let cfg = canonical();
    // Raw formula would be 40 + 10 * 999 = 10030.0 → clamp to 100.0.
    assert!((cfg.coverage_percent(1000) - 100.0).abs() < f32::EPSILON);
}

// ── Behavior 13 — cap applies to oversized base configs ──────────────────────

#[test]
fn coverage_percent_cap_applies_at_stack_one_for_oversized_base() {
    let cfg = TetherConfig {
        base_damage:        25.0,
        damage_per_level:   10.0,
        base_coverage:      150.0,
        coverage_per_level: 0.0,
    };
    // base alone exceeds cap → clamp to 100.0 even at stack 1.
    assert!((cfg.coverage_percent(1) - 100.0).abs() < f32::EPSILON);
}
