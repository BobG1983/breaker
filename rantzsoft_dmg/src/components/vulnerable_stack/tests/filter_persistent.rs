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
