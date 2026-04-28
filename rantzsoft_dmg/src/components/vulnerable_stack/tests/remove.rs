use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

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
    assert_f32_eq(stack.aggregate_persistent(None), 4.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 72: `remove_by_source` for absent source is a no-op ──

#[test]
fn remove_by_source_absent_is_noop() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("m:a"), 1.5);
    stack.remove_by_source(&SourceId::from("m:absent"));
    assert_f32_eq(stack.aggregate_persistent(None), 1.5);
}

#[test]
fn remove_by_source_on_default_stack_does_not_panic() {
    // Edge case for Behavior 72: remove on a default-constructed stack
    // must not panic — aggregate remains 1.0 afterwards.
    let mut stack = VulnerableStack::default();
    stack.remove_by_source(&SourceId::from("m:anything"));
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}
