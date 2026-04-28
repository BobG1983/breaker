use bevy::prelude::*;

use super::{super::component::DamageBoostStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 56: `add` appends a single `(SourceId, f32)` entry ──

#[test]
fn add_appends_single_entry_to_persistent() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.5);
    assert_f32_eq(stack.aggregate_persistent(None), 2.5);
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
    assert_f32_eq(stack.aggregate_persistent(None), 32.0);
}

#[test]
fn interleaved_sources_produce_product_of_all_entries() {
    // Edge case for Behavior 57: interleaved sources do not cross-collapse.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add(SourceId::from("src:beta"), 3.0);
    stack.add(SourceId::from("src:alpha"), 2.0);
    assert_f32_eq(stack.aggregate_persistent(None), 12.0);
}

// ── Behavior 58: mixed sources and mixed multipliers multiply all entries ──

#[test]
fn mixed_sources_and_multipliers_multiply_all_entries() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add(SourceId::from("src:beta"), 3.0);
    stack.add(SourceId::from("src:gamma"), 0.5);
    assert_f32_eq(stack.aggregate_persistent(None), 3.0);
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
    assert_f32_eq(forward.aggregate_persistent(None), 3.0);
    assert_f32_eq(reverse.aggregate_persistent(None), 3.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 25.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(stack.is_empty());
}

// ── Behavior 60: `remove_by_source` for absent source is a no-op ──

#[test]
fn remove_by_source_absent_is_noop() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.remove_by_source(&SourceId::from("src:nonexistent"));
    assert_f32_eq(stack.aggregate_persistent(None), 2.0);
}

#[test]
fn remove_by_source_on_default_stack_does_not_panic() {
    // Edge case for Behavior 60: remove on a default-constructed stack
    // must not panic — aggregate remains 1.0 afterwards.
    let mut stack = DamageBoostStack::default();
    stack.remove_by_source(&SourceId::from("src:anything"));
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

#[test]
fn aggregate_persistent_with_only_one_shots_returns_one() {
    // Edge case for Behavior 62: one_shots does NOT contribute to
    // aggregate_persistent.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(9.9);
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

// ── Behavior 63: `aggregate_and_consume_one_shots` returns product + clears ──

#[test]
fn consume_one_shots_returns_product_then_clears() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    stack.add_one_shot(4.0);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 24.0);
    // Second call returns 1.0 — queue is empty after consumption.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
}

#[test]
fn consume_one_shots_on_default_stack_returns_one() {
    // Edge case for Behavior 63: calling consume on a default stack
    // (no one_shots) returns 1.0 immediately without panicking.
    let mut stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
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
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert_f32_eq(stack.aggregate_persistent(None), 7.0);
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
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert!(stack.is_empty());
    // Observable proxy that persistent was actually drained.
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
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
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

// ── Behavior 65: `Default::default()` produces an empty stack ──

#[test]
fn default_produces_empty_stack() {
    let stack = DamageBoostStack::default();
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

#[test]
fn default_consume_one_shots_returns_one_and_leaves_empty() {
    // Edge case for Behavior 65: running consume on the default stack
    // returns 1.0 and leaves it empty.
    let mut stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
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
