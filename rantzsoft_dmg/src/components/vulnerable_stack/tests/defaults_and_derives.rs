use bevy::prelude::*;

use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

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
