use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 1: filterless persistent entry applies to a matched emission ──

#[test]
fn filterless_persistent_entry_applies_to_matched_emission() {
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.5);
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        1.5,
    );
}

#[test]
fn filterless_persistent_entry_applies_to_none_emission() {
    // Edge case for Behavior 1: filterless entry applies regardless of
    // emission source — including a `None` emission.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 1.5);
    assert_f32_eq(stack.aggregate_persistent(None), 1.5);
}

// ── Behavior 2: filtered persistent entry applies to a matching emission ──

#[test]
fn filtered_persistent_entry_applies_to_matching_emission() {
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
}

#[test]
fn two_filtered_persistent_entries_with_same_filter_combine_multiplicatively() {
    // Edge case for Behavior 2: two filtered entries with the same filter
    // combine multiplicatively (Vec semantics — no collapse).
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        4.0,
    );
}

// ── Behavior 3: filtered persistent entry does NOT apply to non-matching
//    emission ──

#[test]
fn filtered_persistent_entry_does_not_apply_to_non_matching_emission() {
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
}

#[test]
fn filtered_persistent_entry_match_is_case_sensitive() {
    // Edge case for Behavior 3: case-sensitive non-match.
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("hazard:Diffusion"))),
        1.0,
    );
}

// ── Behavior 4: filtered persistent entry does NOT apply to a `None`
//    emission ──

#[test]
fn filtered_persistent_entry_does_not_apply_to_none_emission() {
    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

#[test]
fn filtered_and_filterless_persistent_with_none_emission_returns_filterless_only() {
    // Edge case for Behavior 4: a stack with one filterless and one
    // filtered entry returns the filterless multiplier on a `None`
    // emission — only filterless contributes.
    let mut stack = VulnerableStack::default();
    stack.add(SourceId::from("mark:fragility"), 3.0);
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    assert_f32_eq(stack.aggregate_persistent(None), 3.0);
}

// ── Behavior 5: filterless one-shot applies to a matched emission and is
//    consumed ──

#[test]
fn filterless_one_shot_applies_to_matched_emission_and_is_consumed() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.5);
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        1.5,
    );
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn filterless_one_shot_is_consumed_by_none_emission() {
    // Edge case for Behavior 5: filterless one-shot is consumed by a
    // `None`-emission call.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(1.5);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.5);
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
}

// ── Behavior 6: filtered one-shot applies to a matching emission and is
//    consumed ──

#[test]
fn filtered_one_shot_applies_to_matching_emission_and_is_consumed() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        1.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn two_filtered_one_shots_with_same_filter_combine_and_drain_together() {
    // Edge case for Behavior 6: two filtered one-shots with the same filter
    // combine multiplicatively — `2.0 * 2.0 = 4.0`.
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        4.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 7: filtered one-shot does NOT apply to non-matching emission
//    AND is NOT consumed ──

#[test]
fn filtered_one_shot_does_not_apply_or_consume_on_non_matching_emission() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    // Non-matching emission: returns 1.0 and does NOT consume.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(!stack.is_empty());
    // Follow-up matching emission still drains it.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn non_matching_call_with_mixed_one_shots_drains_only_filterless() {
    // Edge case for Behavior 7: two filtered one-shots (multipliers 2.0 +
    // 3.0, both filtered to "hazard:diffusion") and one filterless one-shot
    // (5.0). Calling with non-matching emission returns 5.0 (filterless
    // contributes and is consumed; both filtered are skipped).
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    stack.add_one_shot_filtered(3.0, SourceId::from("hazard:diffusion"));
    stack.add_one_shot(5.0);

    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        5.0,
    );
    assert!(!stack.is_empty());
    // Follow-up matching call drains both filtered entries: 2.0 * 3.0 = 6.0.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        6.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 8: filtered one-shot does NOT apply to a `None` emission AND
//    is NOT consumed ──

#[test]
fn filtered_one_shot_does_not_apply_or_consume_on_none_emission() {
    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
    assert!(!stack.is_empty());
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn filtered_and_filterless_one_shots_with_none_emission_drains_only_filterless() {
    // Edge case for Behavior 8: filterless one-shot AND filtered one-shot
    // both present. `None`-emission call returns 5.0 (only filterless
    // applies and is consumed); follow-up matching call returns 2.0
    // (filtered survived the `None` call and is now drained).
    let mut stack = VulnerableStack::default();
    stack.add_one_shot(5.0);
    stack.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));

    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 5.0);
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 9: `add_filtered` distinguishes from `add` via aggregate by
//    emission source ──

#[test]
fn add_filtered_distinguishes_from_add_via_aggregate_on_non_matching_emission() {
    let mut with_filter = VulnerableStack::default();
    with_filter.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    let mut without_filter = VulnerableStack::default();
    without_filter.add(SourceId::from("mark:fragility"), 2.0);

    assert_f32_eq(
        with_filter.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert_f32_eq(
        without_filter.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn add_filtered_and_add_both_apply_when_emission_matches_filter() {
    // Edge case for Behavior 9: a matching emission causes BOTH stacks to
    // return 2.0.
    let mut with_filter = VulnerableStack::default();
    with_filter.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    let mut without_filter = VulnerableStack::default();
    without_filter.add(SourceId::from("mark:fragility"), 2.0);

    assert_f32_eq(
        with_filter.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert_f32_eq(
        without_filter.aggregate_persistent(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
}

// ── Behavior 10: `add_one_shot_filtered` distinguishes from `add_one_shot`
//    via aggregate by emission source ──

#[test]
fn add_one_shot_filtered_distinguishes_from_add_one_shot_on_non_matching_emission() {
    let mut with_filter = VulnerableStack::default();
    with_filter.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    let mut without_filter = VulnerableStack::default();
    without_filter.add_one_shot(2.0);

    assert_f32_eq(
        with_filter.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(!with_filter.is_empty());

    assert_f32_eq(
        without_filter.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
    assert!(without_filter.is_empty());
}

#[test]
fn add_one_shot_filtered_drains_on_follow_up_matching_call() {
    // Edge case for Behavior 10: after a non-matching call leaves the
    // filtered one-shot intact, a matching follow-up consumes it.
    let mut with_filter = VulnerableStack::default();
    with_filter.add_one_shot_filtered(2.0, SourceId::from("hazard:diffusion"));
    let _ = with_filter.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));

    assert_f32_eq(
        with_filter.aggregate_and_consume_one_shots(Some(&SourceId::from("hazard:diffusion"))),
        2.0,
    );
    assert!(with_filter.is_empty());
}

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
