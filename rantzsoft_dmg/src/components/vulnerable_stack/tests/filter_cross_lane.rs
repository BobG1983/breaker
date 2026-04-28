use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 11: `is_empty` is true on a fresh stack and false after any
//    `add_filtered` ──

#[test]
fn is_empty_false_after_add_filtered_persistent() {
    let mut stack = VulnerableStack::default();
    assert!(stack.is_empty());
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        1.5,
        SourceId::from("hazard:diffusion"),
    );
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_false_after_add_one_shot_filtered() {
    // Edge case for Behavior 11: a stack with only a filtered one-shot is
    // also non-empty.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(1.5, SourceId::from("hazard:diffusion"));
    assert!(!stack.is_empty());
}

// ── Behavior 12: `aggregate_one_shots(emission_source)` peek-only is
//    filter-aware AND never drains ──

#[test]
fn aggregate_one_shots_peek_is_filter_aware_and_never_drains() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    stack.add_one_shot(3.0);

    // Both apply when emission matches the filter.
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        6.0,
    );
    assert!(!stack.is_empty());
    // Repeat peek returns the same value.
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        6.0,
    );
    // Follow-up consume drains everything that applies.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        6.0,
    );
}

#[test]
fn aggregate_one_shots_peek_with_non_matching_emission_returns_filterless_only() {
    // Edge case for Behavior 12: non-matching emission returns 3.0
    // (filterless contributes, filtered skipped) and remains non-mutating.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    stack.add_one_shot(3.0);

    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout"))),
        3.0,
    );
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout"))),
        3.0,
    );
    assert!(!stack.is_empty());
}

// ── Behavior 13: empty stack returns 1.0 for every aggregate call ──

#[test]
fn empty_stack_returns_one_for_every_aggregate_call_regardless_of_emission_source() {
    let mut stack = VulnerableStack::default();
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(stack.is_empty());
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert!(stack.is_empty());
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert!(stack.is_empty());
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
    assert!(stack.is_empty());
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 14: `remove_by_source` removes filtered AND filterless
//    persistent entries with the same source key ──

#[test]
fn remove_by_source_removes_filtered_and_filterless_persistent_entries() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 2.0);
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        3.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.remove_by_source(&SourceId::from("mark:fragility"));

    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(stack.is_empty());
}

#[test]
fn remove_by_source_removes_only_filtered_persistent_entry_when_alone() {
    // Edge case for Behavior 14: stack with ONLY a filtered entry is also
    // emptied by `remove_by_source` matching the entry's `source` key.
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.remove_by_source(&SourceId::from("mark:fragility"));
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
}

// ── Behavior 15: `remove_by_source` does NOT touch the one-shot lane ──

#[test]
fn remove_by_source_does_not_touch_one_shot_lane() {
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.add_one_shot_filtered(3.0, SourceId::from("hazard:diffusion"));
    stack.remove_by_source(&SourceId::from("mark:fragility"));

    // Persistent removed.
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    // One-shot lane untouched — drains normally.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        3.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn remove_by_source_with_persistent_pair_leaves_filtered_one_shot_intact() {
    // Edge case for Behavior 15: two persistent entries (one filtered, one
    // filterless) sharing source "mark:fragility", PLUS a filtered one-shot
    // under filter "hazard:diffusion". `remove_by_source("mark:fragility")`
    // empties the persistent lane but leaves the one-shot intact.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 2.0);
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        3.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.add_one_shot_filtered(4.0, SourceId::from("hazard:diffusion"));
    stack.remove_by_source(&SourceId::from("mark:fragility"));

    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        4.0,
    );
    assert!(stack.is_empty());
}
