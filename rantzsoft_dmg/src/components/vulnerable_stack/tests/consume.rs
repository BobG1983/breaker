use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

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
