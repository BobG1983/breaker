use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 86: `aggregate_one_shots` returns 1.0 when empty ──

#[test]
fn aggregate_one_shots_empty_returns_one() {
    let stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

#[test]
fn aggregate_one_shots_on_default_stack_does_not_panic_and_is_idempotent() {
    // Edge case for Behavior 86: a default-constructed stack must NOT
    // panic when called. Two consecutive calls both return 1.0.
    let stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

// ── Behavior 87: single one-shot returns the value ──

#[test]
fn aggregate_one_shots_single_value_returns_that_value() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(2.5);
    assert_f32_eq(stack.aggregate_one_shots(), 2.5);
}

#[test]
fn aggregate_one_shots_single_identity_value_returns_one_but_is_not_empty() {
    // Edge case for Behavior 87: a one-shot of identity 1.0 returns 1.0
    // AND is_empty() returns false.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.0);
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 88: multiple one-shots return the product ──

#[test]
fn aggregate_one_shots_multiple_values_returns_product() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(1.5);
    stack.add_one_shot(3.0);
    // 2.0 * 1.5 * 3.0 = 9.0.
    assert_f32_eq(stack.aggregate_one_shots(), 9.0);
}

#[test]
fn aggregate_one_shots_is_order_independent() {
    // Edge case for Behavior 88: order independence — build a reverse-order
    // stack; both pin to 9.0 independently.
    let mut forward = VulnerableStack::default();
    forward.add_one_shot(2.0);
    forward.add_one_shot(1.5);
    forward.add_one_shot(3.0);

    let mut reverse = VulnerableStack::default();
    reverse.add_one_shot(3.0);
    reverse.add_one_shot(1.5);
    reverse.add_one_shot(2.0);

    assert_f32_eq(forward.aggregate_one_shots(), 9.0);
    assert_f32_eq(reverse.aggregate_one_shots(), 9.0);
}

// ── Behavior 89: peek does not consume ──

#[test]
fn aggregate_one_shots_does_not_consume_across_repeated_calls() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    // Both peek calls return 6.0.
    assert_f32_eq(stack.aggregate_one_shots(), 6.0);
    assert_f32_eq(stack.aggregate_one_shots(), 6.0);
    // A subsequent consume ALSO returns 6.0 — confirms the peek did not
    // drain.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(), 6.0);
}

#[test]
fn aggregate_one_shots_after_consume_returns_one() {
    // Edge case for Behavior 89: after the consume, a follow-up peek
    // returns 1.0.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    let _ = stack.aggregate_and_consume_one_shots();
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

// ── Behavior 90: `aggregate_one_shots` ignores the persistent lane ──

#[test]
fn aggregate_one_shots_ignores_persistent_lane() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 5.0);
    assert_f32_eq(stack.aggregate_one_shots(), 1.0);
}

#[test]
fn aggregate_one_shots_isolated_from_persistent_when_both_populated() {
    // Edge case for Behavior 90: persistent + one-shot — peek of one_shots
    // returns only 2.0 AND persistent lane still returns 5.0.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 5.0);
    stack.add_one_shot(2.0);
    assert_f32_eq(stack.aggregate_one_shots(), 2.0);
    assert_f32_eq(stack.aggregate_persistent(), 5.0);
}

// ── Behavior 91: symmetric peek of both lanes is non-mutating ──

#[test]
fn aggregate_one_shots_and_persistent_are_each_isolated() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 6.0);
    stack.add_one_shot(13.0);
    assert_f32_eq(stack.aggregate_one_shots(), 13.0);
    assert_f32_eq(stack.aggregate_persistent(), 6.0);
}

#[test]
fn aggregate_persistent_then_one_shots_are_each_isolated() {
    // Edge case for Behavior 91: reverse call order — same results.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 6.0);
    stack.add_one_shot(13.0);
    assert_f32_eq(stack.aggregate_persistent(), 6.0);
    assert_f32_eq(stack.aggregate_one_shots(), 13.0);
}

#[test]
fn aggregate_one_shots_idempotent_after_cross_method_calls() {
    // Edge case for Behavior 91: a second `aggregate_one_shots()` after
    // both calls still returns 13.0.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 6.0);
    stack.add_one_shot(13.0);
    let _ = stack.aggregate_one_shots();
    let _ = stack.aggregate_persistent();
    assert_f32_eq(stack.aggregate_one_shots(), 13.0);
}
