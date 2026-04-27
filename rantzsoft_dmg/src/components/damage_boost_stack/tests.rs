use bevy::prelude::*;

use super::component::DamageBoostStack;
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

// ── Behavior 56: `add` appends a single `(SourceId, f32)` entry ──

#[test]
fn add_appends_single_entry_to_persistent() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.5);
    assert_f32_eq(stack.aggregate_persistent(), 2.5);
    assert!(!stack.is_empty());
}

#[test]
fn add_on_default_stack_does_not_panic() {
    // Edge case for Behavior 56: calling `add` on a default-constructed
    // stack must succeed without panicking (no pre-reservation
    // requirement).
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 57: same source added five times produces five entries ──

#[test]
fn same_source_added_five_times_aggregates_to_mult_pow_five() {
    let mut stack = DamageBoostStack::default();
    for _ in 0..5 {
        stack.add(SourceId::from("src:alpha"), 2.0);
    }
    // Pins the Vec semantic: if a future implementer swaps to
    // `HashMap<SourceId, f32>`, this aggregate would collapse to 2.0.
    assert_f32_eq(stack.aggregate_persistent(), 32.0);
}

#[test]
fn interleaved_sources_produce_product_of_all_entries() {
    // Edge case for Behavior 57: interleaved sources do not cross-collapse.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add(SourceId::from("src:beta"), 3.0);
    stack.add(SourceId::from("src:alpha"), 2.0);
    assert_f32_eq(stack.aggregate_persistent(), 12.0);
}

// ── Behavior 58: mixed sources and mixed multipliers multiply all entries ──

#[test]
fn mixed_sources_and_multipliers_multiply_all_entries() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add(SourceId::from("src:beta"), 3.0);
    stack.add(SourceId::from("src:gamma"), 0.5);
    assert_f32_eq(stack.aggregate_persistent(), 3.0);
}

#[test]
fn aggregate_is_order_independent() {
    // Edge case for Behavior 58: order of insertion does not change the
    // aggregate (multiplication is commutative).
    let mut forward = DamageBoostStack::default();
    forward.add(SourceId::from("src:alpha"), 2.0);
    forward.add(SourceId::from("src:beta"), 3.0);
    forward.add(SourceId::from("src:gamma"), 0.5);

    let mut reverse = DamageBoostStack::default();
    reverse.add(SourceId::from("src:gamma"), 0.5);
    reverse.add(SourceId::from("src:beta"), 3.0);
    reverse.add(SourceId::from("src:alpha"), 2.0);

    // Assert against a concrete expected value (2.0 * 3.0 * 0.5 = 3.0)
    // on BOTH sides independently. Comparing forward vs reverse directly
    // would pass trivially against the RED stub (both return 0.0), so we
    // pin each side to 3.0 to keep the RED-gate signal meaningful.
    assert_f32_eq(forward.aggregate_persistent(), 3.0);
    assert_f32_eq(reverse.aggregate_persistent(), 3.0);
}

// ── Behavior 59: `remove_by_source` removes ALL matching entries ──

#[test]
fn remove_by_source_removes_every_matching_entry() {
    let mut stack = DamageBoostStack::default();
    for _ in 0..3 {
        stack.add(SourceId::from("src:alpha"), 2.0);
    }
    for _ in 0..2 {
        stack.add(SourceId::from("src:beta"), 5.0);
    }
    stack.remove_by_source(&SourceId::from("src:alpha"));
    // Only the two beta 5.0 entries survive: 5.0 * 5.0 = 25.0.
    assert_f32_eq(stack.aggregate_persistent(), 25.0);
}

#[test]
fn remove_by_source_of_every_source_leaves_stack_empty() {
    // Edge case for Behavior 59: removing every source leaves the stack
    // empty and aggregate back to 1.0.
    let mut stack = DamageBoostStack::default();
    for _ in 0..3 {
        stack.add(SourceId::from("src:alpha"), 2.0);
    }
    for _ in 0..2 {
        stack.add(SourceId::from("src:beta"), 5.0);
    }
    stack.remove_by_source(&SourceId::from("src:alpha"));
    stack.remove_by_source(&SourceId::from("src:beta"));
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 60: `remove_by_source` for absent source is a no-op ──

#[test]
fn remove_by_source_absent_is_noop() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.remove_by_source(&SourceId::from("src:nonexistent"));
    assert_f32_eq(stack.aggregate_persistent(), 2.0);
}

#[test]
fn remove_by_source_on_default_stack_does_not_panic() {
    // Edge case for Behavior 60: remove on a default-constructed stack
    // must not panic — aggregate remains 1.0 afterwards.
    let mut stack = DamageBoostStack::default();
    stack.remove_by_source(&SourceId::from("src:anything"));
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 61: `add_one_shot` appends a bare multiplier to `one_shots` ──

#[test]
fn add_one_shot_appends_to_one_shots_only() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    // one_shots is non-empty.
    assert!(!stack.is_empty());
    // persistent is still empty — aggregate_persistent returns 1.0.
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn add_one_shot_with_identity_multiplier_still_appends() {
    // Edge case for Behavior 61: multiplier of 1.0 is still an appended
    // entry; `is_empty` becomes false.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 62: `aggregate_persistent` returns 1.0 when empty ──

#[test]
fn aggregate_persistent_empty_returns_one() {
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn aggregate_persistent_with_only_one_shots_returns_one() {
    // Edge case for Behavior 62: one_shots does NOT contribute to
    // aggregate_persistent.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(9.9);
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 63: `aggregate_and_consume_one_shots` returns product + clears ──

#[test]
fn consume_one_shots_returns_product_then_clears() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    stack.add_one_shot(4.0);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 24.0);
    // Second call returns 1.0 — queue is empty after consumption.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn consume_one_shots_on_default_stack_returns_one() {
    // Edge case for Behavior 63: calling consume on a default stack
    // (no one_shots) returns 1.0 immediately without panicking.
    let mut stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
}

#[test]
fn consume_one_shots_does_not_touch_persistent() {
    // Edge case for Behavior 63: add_one_shot-then-consume must NOT
    // touch persistent. Pre-populate persistent with add(source, 7.0),
    // then call consume — afterwards, aggregate_persistent must still
    // equal 7.0.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    let _ = stack.aggregate_and_consume_one_shots();
    assert_f32_eq(stack.aggregate_persistent(), 7.0);
}

// ── Behavior 64: `is_empty` is true iff BOTH lanes are empty ──

#[test]
fn is_empty_default_stack_is_true() {
    assert!(DamageBoostStack::default().is_empty());
}

#[test]
fn is_empty_after_add_only_is_false() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_after_add_one_shot_only_is_false() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_after_both_populated_then_both_drained_is_true() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add_one_shot(3.0);
    stack.remove_by_source(&SourceId::from("src:alpha"));
    let _ = stack.aggregate_and_consume_one_shots();
    assert!(stack.is_empty());
    // Observable proxy that persistent was actually drained.
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn is_empty_after_add_then_remove_returns_to_true() {
    // Edge case for Behavior 64: add + remove cycle returns is_empty to
    // true AND aggregate_persistent to 1.0 (observable proxy that the
    // entry was removed, not merely marked).
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.remove_by_source(&SourceId::from("src:alpha"));
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

// ── Behavior 65: `Default::default()` produces an empty stack ──

#[test]
fn default_produces_empty_stack() {
    let stack = DamageBoostStack::default();
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_persistent(), 1.0);
}

#[test]
fn default_consume_one_shots_returns_one_and_leaves_empty() {
    // Edge case for Behavior 65: running consume on the default stack
    // returns 1.0 and leaves it empty.
    let mut stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 66: `DamageBoostStack` is a Bevy `Component` ──

#[test]
fn spawns_as_component_in_bare_world() {
    let mut world = World::new();
    let e = world.spawn(DamageBoostStack::default()).id();
    assert!(world.get::<DamageBoostStack>(e).is_some());
}

#[test]
fn empty_spawn_does_not_get_damage_boost_stack() {
    let mut world = World::new();
    let e = world.spawn_empty().id();
    assert!(world.get::<DamageBoostStack>(e).is_none());
}

// ── Behavior 67: `DamageBoostStack` derives `Debug` ──

#[test]
fn debug_format_default_stack_contains_type_name() {
    let stack = DamageBoostStack::default();
    let s = format!("{stack:?}");
    assert!(!s.is_empty());
    assert!(s.contains("DamageBoostStack"));
}

#[test]
fn debug_format_populated_stack_is_non_empty() {
    // Edge case for Behavior 67: formatting a populated stack also
    // succeeds and returns a non-empty string.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add_one_shot(3.0);
    let s = format!("{stack:?}");
    assert!(!s.is_empty());
}

// ── Behavior 80: `aggregate_one_shots` returns 1.0 when empty ──

#[test]
fn aggregate_one_shots_empty_returns_one() {
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

#[test]
fn aggregate_one_shots_on_default_stack_does_not_panic_and_is_idempotent() {
    // Edge case for Behavior 80: a default-constructed stack must NOT
    // panic when called. Two consecutive calls both return 1.0
    // (idempotent on empty input).
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

// ── Behavior 81: single one-shot returns the value ──

#[test]
fn aggregate_one_shots_single_value_returns_that_value() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.5);
    assert_f32_eq(stack.aggregate_one_shots(), 2.5);
}

#[test]
fn aggregate_one_shots_single_identity_value_returns_one_but_is_not_empty() {
    // Edge case for Behavior 81: a one-shot of identity 1.0 returns 1.0
    // from `aggregate_one_shots()` — same value as the empty case, but
    // `is_empty()` is false. Pins the lane semantic.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(1.0);
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 82: multiple one-shots return the product ──

#[test]
fn aggregate_one_shots_multiple_values_returns_product() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(1.5);
    stack.add_one_shot(3.0);
    // 2.0 * 1.5 * 3.0 = 9.0.
    assert_f32_eq(stack.aggregate_one_shots(), 9.0);
}

#[test]
fn aggregate_one_shots_is_order_independent() {
    // Edge case for Behavior 82: order independence. Build a second
    // stack with the same three values appended in reverse order.
    // Pin BOTH stacks to the concrete value 9.0 independently —
    // do NOT compare forward vs reverse (would pass a constant-RED stub).
    let mut forward = DamageBoostStack::default();
    forward.add_one_shot(2.0);
    forward.add_one_shot(1.5);
    forward.add_one_shot(3.0);

    let mut reverse = DamageBoostStack::default();
    reverse.add_one_shot(3.0);
    reverse.add_one_shot(1.5);
    reverse.add_one_shot(2.0);

    assert_f32_eq(forward.aggregate_one_shots(), 9.0);
    assert_f32_eq(reverse.aggregate_one_shots(), 9.0);
}

// ── Behavior 83: peek does not consume — multiple calls return same value ──

#[test]
fn aggregate_one_shots_does_not_consume_across_repeated_calls() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    // Both peek calls return 6.0.
    assert_f32_eq(stack.aggregate_one_shots(), 6.0);
    assert_f32_eq(stack.aggregate_one_shots(), 6.0);
    // A subsequent consume ALSO returns 6.0 — observable proof the peek
    // calls did NOT drain the lane.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 6.0);
}

#[test]
fn aggregate_one_shots_after_consume_returns_one() {
    // Edge case for Behavior 83: after the consume, a follow-up peek
    // returns 1.0 (lane is now empty — confirms consume drained AND
    // that the new peek method correctly observes the post-drain
    // empty state).
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    let _ = stack.aggregate_and_consume_one_shots();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

// ── Behavior 84: `aggregate_one_shots` ignores the persistent lane ──

#[test]
fn aggregate_one_shots_ignores_persistent_lane() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    // Persistent only — one-shot lane is untouched.
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

#[test]
fn aggregate_one_shots_isolated_from_persistent_when_both_populated() {
    // Edge case for Behavior 84: persistent + one-shot — peek of one_shots
    // returns only the one-shot product (2.0); persistent lane still
    // returns 5.0 (peek did not corrupt the persistent lane).
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    stack.add_one_shot(2.0);
    assert_f32_eq(stack.aggregate_one_shots(), 2.0);
    assert_f32_eq(stack.aggregate_persistent(), 5.0);
}

// ── Behavior 85: symmetric counterpart to `aggregate_persistent` does not pollute it ──

#[test]
fn aggregate_one_shots_and_persistent_are_each_isolated() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    // Call peek of one_shots, then persistent — neither lane is mutated.
    assert_f32_eq(stack.aggregate_one_shots(), 11.0);
    assert_f32_eq(stack.aggregate_persistent(), 7.0);
}

#[test]
fn aggregate_persistent_then_one_shots_are_each_isolated() {
    // Edge case for Behavior 85: reverse the call order.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    assert_f32_eq(stack.aggregate_persistent(), 7.0);
    assert_f32_eq(stack.aggregate_one_shots(), 11.0);
}

#[test]
fn aggregate_one_shots_idempotent_after_cross_method_calls() {
    // Edge case for Behavior 85: call peek of one_shots again after
    // both calls — still 11.0. Pins idempotence under cross-method
    // calls.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    let _ = stack.aggregate_one_shots();
    let _ = stack.aggregate_persistent();
    assert_f32_eq(stack.aggregate_one_shots(), 11.0);
}
