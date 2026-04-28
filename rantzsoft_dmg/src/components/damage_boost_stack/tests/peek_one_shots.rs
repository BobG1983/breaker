use super::{super::component::DamageBoostStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 80: `aggregate_one_shots` returns 1.0 when empty ──

#[test]
fn aggregate_one_shots_empty_returns_one() {
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

#[test]
fn aggregate_one_shots_on_default_stack_does_not_panic_and_is_idempotent() {
    // Edge case for Behavior 80: a default-constructed stack must NOT
    // panic when called. Two consecutive calls both return 1.0
    // (idempotent on empty input).
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

// ── Behavior 81: single one-shot returns the value ──

#[test]
fn aggregate_one_shots_single_value_returns_that_value() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.5);
    assert_f32_eq(stack.aggregate_one_shots(None), 2.5);
}

#[test]
fn aggregate_one_shots_single_identity_value_returns_one_but_is_not_empty() {
    // Edge case for Behavior 81: a one-shot of identity 1.0 returns 1.0
    // from `aggregate_one_shots()` — same value as the empty case, but
    // `is_empty()` is false. Pins the lane semantic.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(1.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
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
    assert_f32_eq(stack.aggregate_one_shots(None), 9.0);
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

    assert_f32_eq(forward.aggregate_one_shots(None), 9.0);
    assert_f32_eq(reverse.aggregate_one_shots(None), 9.0);
}

// ── Behavior 83: peek does not consume — multiple calls return same value ──

#[test]
fn aggregate_one_shots_does_not_consume_across_repeated_calls() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    // Both peek calls return 6.0.
    assert_f32_eq(stack.aggregate_one_shots(None), 6.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 6.0);
    // A subsequent consume ALSO returns 6.0 — observable proof the peek
    // calls did NOT drain the lane.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 6.0);
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
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

// ── Behavior 84: `aggregate_one_shots` ignores the persistent lane ──

#[test]
fn aggregate_one_shots_ignores_persistent_lane() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    // Persistent only — one-shot lane is untouched.
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

#[test]
fn aggregate_one_shots_isolated_from_persistent_when_both_populated() {
    // Edge case for Behavior 84: persistent + one-shot — peek of one_shots
    // returns only the one-shot product (2.0); persistent lane still
    // returns 5.0 (peek did not corrupt the persistent lane).
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    stack.add_one_shot(2.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 2.0);
    assert_f32_eq(stack.aggregate_persistent(None), 5.0);
}

// ── Behavior 85: symmetric counterpart to `aggregate_persistent` does not pollute it ──

#[test]
fn aggregate_one_shots_and_persistent_are_each_isolated() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    // Call peek of one_shots, then persistent — neither lane is mutated.
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
    assert_f32_eq(stack.aggregate_persistent(None), 7.0);
}

#[test]
fn aggregate_persistent_then_one_shots_are_each_isolated() {
    // Edge case for Behavior 85: reverse the call order.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    assert_f32_eq(stack.aggregate_persistent(None), 7.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
}

#[test]
fn aggregate_one_shots_idempotent_after_cross_method_calls() {
    // Edge case for Behavior 85: call peek of one_shots again after
    // both calls — still 11.0. Pins idempotence under cross-method
    // calls.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    let _ = stack.aggregate_one_shots(None);
    let _ = stack.aggregate_persistent(None);
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
}
