use super::{super::system::*, helpers::*};

// No bevy imports needed — these tests only exercise pure config methods.

// ── splits_for formula ────────────────────────────────────────────────

// Behavior 1 — stack 0 returns 0 (identity).
#[test]
fn splits_zero_stacks_is_zero() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    };
    assert_eq!(cfg.splits_for(0), 0);
}

#[test]
fn splits_zero_stacks_is_zero_even_with_large_base() {
    let cfg = FractureConfig {
        base_splits:      100,
        per_level_splits: 50,
    };
    assert_eq!(cfg.splits_for(0), 0);
}

// Behavior 2 — stack 1 returns base_splits.
#[test]
fn splits_stack_one_is_base() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    };
    assert_eq!(cfg.splits_for(1), 2);
}

#[test]
fn splits_stack_one_ignores_per_level_multiplier() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 999,
    };
    assert_eq!(cfg.splits_for(1), 2);
}

// Behavior 3 — stack 2 returns base + per_level.
#[test]
fn splits_stack_two_is_base_plus_per_level() {
    let cfg = canonical_config();
    assert_eq!(cfg.splits_for(2), 3);
}

#[test]
fn splits_stack_two_with_zero_base_uses_per_level() {
    let cfg = FractureConfig {
        base_splits:      0,
        per_level_splits: 3,
    };
    assert_eq!(cfg.splits_for(2), 3);
}

// Behavior 4 — stack 3 returns base + 2*per_level.
#[test]
fn splits_stack_three_adds_two_levels() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    };
    // 2 + 1 * 2 = 4
    assert_eq!(cfg.splits_for(3), 4);
}

#[test]
fn splits_stack_three_with_zero_base_still_grows() {
    let cfg = FractureConfig {
        base_splits:      0,
        per_level_splits: 2,
    };
    // 0 + 2 * 2 = 4
    assert_eq!(cfg.splits_for(3), 4);
}

// Behavior 5 — stack 10 capped at 4.
#[test]
fn splits_are_capped_at_offset_count() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    };
    // Stack 10 would be 11 but we have only 4 offsets.
    assert_eq!(cfg.splits_for(10), 4);
}

#[test]
fn splits_constant_when_per_level_is_zero() {
    let cfg = FractureConfig {
        base_splits:      2,
        per_level_splits: 0,
    };
    // per_level=0 → constant at base_splits across all stacks ≥ 1; cap not engaged.
    assert_eq!(cfg.splits_for(10), 2);
}

// Behavior 6 — base=0, per_level=0 → 0 at all stacks.
#[test]
fn splits_zero_config_is_zero_at_all_stacks() {
    let cfg = FractureConfig {
        base_splits:      0,
        per_level_splits: 0,
    };
    for k in [0u32, 1, 2, 3, 5, 10] {
        assert_eq!(cfg.splits_for(k), 0, "k={k}");
    }
}

#[test]
fn splits_zero_base_with_large_per_level_stack_one_is_zero_stack_two_caps() {
    let cfg = FractureConfig {
        base_splits:      0,
        per_level_splits: 100,
    };
    // Stack 1: 0 + 100 * 0 = 0 (extra = 0 short-circuits).
    assert_eq!(cfg.splits_for(1), 0);
    // Stack 2: 0 + 100 * 1 = 100, cap at 4.
    assert_eq!(cfg.splits_for(2), 4);
}

// Behavior 7 — base=4 stack=1 at cap boundary (not over).
#[test]
fn splits_base_four_stack_one_is_exactly_four() {
    let cfg = FractureConfig {
        base_splits:      4,
        per_level_splits: 0,
    };
    // raw == 4: does NOT trigger `raw > 4` (strict inequality).
    assert_eq!(cfg.splits_for(1), 4);
}

#[test]
fn splits_base_five_stack_one_is_clamped_to_four() {
    let cfg = FractureConfig {
        base_splits:      5,
        per_level_splits: 0,
    };
    // raw == 5: triggers `raw > 4` cap.
    assert_eq!(cfg.splits_for(1), 4);
}

// Behavior 8 — u32::MAX overflow safety.
#[test]
fn splits_overflow_saturates_and_caps_at_four() {
    let cfg = FractureConfig {
        base_splits:      u32::MAX,
        per_level_splits: u32::MAX,
    };
    assert_eq!(cfg.splits_for(u32::MAX), 4);
}

#[test]
fn splits_saturating_add_never_wraps_under_cap() {
    let cfg = FractureConfig {
        base_splits:      u32::MAX,
        per_level_splits: 1,
    };
    // Stack 2: extra = 1; mul = 1; add saturates to u32::MAX; cap to 4.
    assert_eq!(cfg.splits_for(2), 4);
}

// Behavior 9 — stack 4 raw=5 clamped to 4.
#[test]
fn splits_stack_four_canonical_clamps_to_four() {
    let cfg = canonical_config();
    // 2 + 1 * 3 = 5, clamped to 4.
    assert_eq!(cfg.splits_for(4), 4);
}

#[test]
fn splits_base_three_per_level_one_stack_two_is_exactly_four() {
    let cfg = FractureConfig {
        base_splits:      3,
        per_level_splits: 1,
    };
    // 3 + 1 * 1 = 4 (NOT clamped — strict `>` means raw=4 passes).
    assert_eq!(cfg.splits_for(2), 4);
}
