use super::{super::system::VulnerableStack, helpers::assert_f32_eq};
use crate::SourceId;

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
