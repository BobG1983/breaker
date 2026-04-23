use bevy::prelude::*;

use super::system::*;
use crate::SourceId;

/// Compares two `f32` values for "equality" without tripping
/// `clippy::float_cmp`. Handles infinity by checking sign and finiteness
/// directly (so we never compute `INF - INF`, which produces NaN).
#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

// ── Behavior 68: `add` appends a single `(SourceId, f32)` entry ──

#[test]
fn add_appends_single_entry_to_persistent() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.5);
    assert_f32_eq(stack.aggregate_persistent(), 1.5);
    assert!(!stack.is_empty());
}

#[test]
fn add_on_default_stack_does_not_panic() {
    // Edge case for Behavior 68: calling `add` on a default-constructed
    // stack must succeed without panicking.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 69: same source added five times produces five entries ──

#[test]
fn same_source_added_five_times_aggregates_to_mult_pow_five() {
    let mut stack = VulnerableStack::default();
    for _ in 0..5 {
        stack.add(SourceId::from("mark:fragility"), 3.0);
    }
    // Pins the Vec semantic: if a future implementer swaps to
    // `HashMap<SourceId, f32>`, this aggregate would collapse to 3.0.
    // 3.0_f32.powi(5) == 243.0.
    assert_f32_eq(stack.aggregate_persistent(), 243.0);
}

#[test]
fn interleaved_sources_produce_product_of_all_entries() {
    // Edge case for Behavior 69: interleaved sources do not cross-collapse.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 2.0);
    stack.add(SourceId::from("m:b"), 1.25);
    stack.add(SourceId::from("m:a"), 2.0);
    assert_f32_eq(stack.aggregate_persistent(), 5.0);
}

// ── Behavior 70: mixed sources and mixed multipliers multiply all entries ──

#[test]
fn mixed_sources_and_multipliers_multiply_all_entries() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.add(SourceId::from("m:b"), 2.0);
    stack.add(SourceId::from("m:c"), 4.0);
    // 1.5 * 2.0 * 4.0 = 12.0.
    assert_f32_eq(stack.aggregate_persistent(), 12.0);
}

#[test]
fn aggregate_is_order_independent() {
    // Edge case for Behavior 70: order of insertion does not change the
    // aggregate (multiplication is commutative).
    let mut forward = VulnerableStack::default();
    forward.add(SourceId::from("m:a"), 1.5);
    forward.add(SourceId::from("m:b"), 2.0);
    forward.add(SourceId::from("m:c"), 4.0);

    let mut reverse = VulnerableStack::default();
    reverse.add(SourceId::from("m:c"), 4.0);
    reverse.add(SourceId::from("m:b"), 2.0);
    reverse.add(SourceId::from("m:a"), 1.5);

    // Assert against a concrete expected value (1.5 * 2.0 * 4.0 = 12.0)
    // on BOTH sides independently. Comparing forward vs reverse directly
    // would pass trivially against the RED stub (both return 0.0), so we
    // pin each side to 12.0 to keep the RED-gate signal meaningful.
    assert_f32_eq(forward.aggregate_persistent(), 12.0);
    assert_f32_eq(reverse.aggregate_persistent(), 12.0);
}

// ── Behavior 71: `remove_by_source` removes ALL matching entries ──

#[test]
fn remove_by_source_removes_every_matching_entry() {
    let mut stack = VulnerableStack::default();
    for _ in 0..3 {
        stack.add(SourceId::from("m:a"), 3.0);
    }
    for _ in 0..2 {
        stack.add(SourceId::from("m:b"), 2.0);
    }
    stack.remove_by_source(&SourceId::from("m:a"));
    // Only the two m:b 2.0 entries survive: 2.0 * 2.0 = 4.0.
    assert_f32_eq(stack.aggregate_persistent(), 4.0);
}

#[test]
fn remove_by_source_of_every_source_leaves_stack_empty() {
    // Edge case for Behavior 71: removing every source leaves the stack
    // empty and aggregate back to 1.0.
    let mut stack = VulnerableStack::default();
    for _ in 0..3 {
        stack.add(SourceId::from("m:a"), 3.0);
    }
    for _ in 0..2 {
        stack.add(SourceId::from("m:b"), 2.0);
    }
    stack.remove_by_source(&SourceId::from("m:a"));
    stack.remove_by_source(&SourceId::from("m:b"));
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 72: `remove_by_source` for absent source is a no-op ──

#[test]
fn remove_by_source_absent_is_noop() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.remove_by_source(&SourceId::from("m:absent"));
    assert_f32_eq(stack.aggregate_persistent(), 1.5);
}

#[test]
fn remove_by_source_on_default_stack_does_not_panic() {
    // Edge case for Behavior 72: remove on a default-constructed stack
    // must not panic — aggregate remains 1.0 afterwards.
    let mut stack = VulnerableStack::default();
    stack.remove_by_source(&SourceId::from("m:anything"));
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 73: `add_one_shot` appends a bare multiplier to `one_shots` ──

#[test]
fn add_one_shot_appends_to_one_shots_only() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(2.5);
    stack.add_one_shot(4.0);
    // one_shots is non-empty.
    assert!(!stack.is_empty());
    // persistent is still empty — aggregate_persistent returns 1.0.
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn add_one_shot_with_identity_multiplier_still_appends() {
    // Edge case for Behavior 73: multiplier of 1.0 is still an appended
    // entry; `is_empty` becomes false.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 74: `aggregate_persistent` returns 1.0 when empty ──

#[test]
fn aggregate_persistent_empty_returns_one() {
    let stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn aggregate_persistent_with_only_one_shots_returns_one() {
    // Edge case for Behavior 74: one_shots does NOT contribute to
    // aggregate_persistent.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(9.9);
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 75: `aggregate_and_consume_one_shots` returns product + clears ──

#[test]
fn consume_one_shots_returns_product_then_clears() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.5);
    stack.add_one_shot(2.0);
    stack.add_one_shot(4.0);
    // 1.5 * 2.0 * 4.0 = 12.0.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 12.0);
    // Second call returns 1.0 — queue is empty after consumption.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn consume_one_shots_on_default_stack_returns_one() {
    // Edge case for Behavior 75: calling consume on a default stack
    // (no one_shots) returns 1.0 immediately without panicking.
    let mut stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn consume_one_shots_does_not_touch_persistent() {
    // Edge case for Behavior 75: add_one_shot-then-consume must NOT
    // touch persistent. Pre-populate persistent with add(source, 6.0),
    // then call consume — afterwards, aggregate_persistent must still
    // equal 6.0.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 6.0);
    stack.add_one_shot(13.0);
    let _ = stack.aggregate_and_consume_one_shots();
    assert_f32_eq(stack.aggregate_persistent(), 6.0);
}

// ── Behavior 76: `is_empty` is true iff BOTH lanes are empty ──

#[test]
fn is_empty_default_stack_is_true() {
    assert!(VulnerableStack::default().is_empty());
}

#[test]
fn is_empty_after_add_only_is_false() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_after_add_one_shot_only_is_false() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.5);
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_after_both_populated_then_both_drained_is_true() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.add_one_shot(2.0);
    stack.remove_by_source(&SourceId::from("m:a"));
    let _ = stack.aggregate_and_consume_one_shots();
    assert!(stack.is_empty());
    // Observable proxy that persistent was actually drained.
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn is_empty_after_add_then_remove_returns_to_true() {
    // Edge case for Behavior 76: add + remove cycle returns is_empty to
    // true AND aggregate_persistent to 1.0 (observable proxy that the
    // entry was removed, not merely marked).
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.remove_by_source(&SourceId::from("m:a"));
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 77: `Default::default()` produces an empty stack ──

#[test]
fn default_produces_empty_stack() {
    let stack = VulnerableStack::default();
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn default_consume_one_shots_returns_one_and_leaves_empty() {
    // Edge case for Behavior 77: running consume on the default stack
    // returns 1.0 and leaves it empty.
    let mut stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 78: `VulnerableStack` is a Bevy `Component` ──

#[test]
fn spawns_as_component_in_bare_world() {
    let mut world = World::new();
    let e = world.spawn(VulnerableStack::default()).id();
    assert!(world.get::<VulnerableStack>(e).is_some());
}

#[test]
fn empty_spawn_does_not_get_vulnerable_stack() {
    let mut world = World::new();
    let e = world.spawn_empty().id();
    assert!(world.get::<VulnerableStack>(e).is_none());
}

// ── Behavior 79: `VulnerableStack` derives `Debug` ──

#[test]
fn debug_format_default_stack_contains_type_name() {
    let stack = VulnerableStack::default();
    let s = format!("{stack:?}");
    assert!(!s.is_empty());
    assert!(s.contains("VulnerableStack"));
}

#[test]
fn debug_format_populated_stack_is_non_empty() {
    // Edge case for Behavior 79: formatting a populated stack also
    // succeeds and returns a non-empty string.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.add_one_shot(2.0);
    let s = format!("{stack:?}");
    assert!(!s.is_empty());
}
