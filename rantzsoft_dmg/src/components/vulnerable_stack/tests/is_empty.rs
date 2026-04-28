use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

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
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert!(stack.is_empty());
    // Observable proxy that persistent was actually drained.
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}
