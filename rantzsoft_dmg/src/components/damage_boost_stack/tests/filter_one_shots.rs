use super::{super::component::DamageBoostStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 94: one-shot — filterless entry × matched emission applies and is consumed ──

#[test]
fn one_shot_filterless_entry_applies_and_is_consumed() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn one_shot_filterless_entry_drained_after_first_consume() {
    // Edge case for Behavior 94: a SECOND consume returns 1.0 (lane
    // drained) AND `is_empty` is `true`.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 95: one-shot — filtered entry × matched emission applies and is consumed ──

#[test]
fn one_shot_filtered_entry_matching_emission_applies_and_is_consumed() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn one_shot_filtered_entry_drained_after_matching_consume() {
    // Edge case for Behavior 95: a SECOND consume returns 1.0; the
    // filtered one-shot was drained on the first matching consume just
    // like a filterless one-shot.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 96: one-shot — filtered entry × wrong-source emission does NOT apply AND is NOT consumed ──

#[test]
fn one_shot_filtered_entry_wrong_source_emission_does_not_apply() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}

#[test]
fn one_shot_filtered_entry_wrong_source_emission_does_not_consume() {
    // Edge case for Behavior 96: critical contract. After a non-matching
    // consume call, the lane is NOT drained — `is_empty` is false AND
    // a follow-up matching consume drains the entry.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:debt_collector")));
    assert!(!stack.is_empty());
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

// ── Behavior 97: one-shot — filtered entry × None emission does NOT apply AND is NOT consumed ──

#[test]
fn one_shot_filtered_entry_none_emission_does_not_apply() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 1.0);
}

#[test]
fn one_shot_filtered_entry_none_emission_does_not_consume() {
    // Edge case for Behavior 97: after a `None`-emission consume, the
    // lane is NOT drained — `is_empty` is false AND a follow-up matching
    // consume drains the entry.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert!(!stack.is_empty());
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

// ── Behavior 98: one-shot — mixed-filter lane drains only matching entries ──

#[test]
fn one_shot_mixed_lane_burnout_consume_drains_only_matching_entries() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
    stack.add_one_shot_filtered(5.0, SourceId::from("protocol:debt_collector"));
    // 2.0 (filterless) * 3.0 (burnout-filtered) = 6.0; the 5.0 entry is
    // filtered to debt_collector and is NOT drained or counted here.
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        6.0,
    );
}

#[test]
fn one_shot_mixed_lane_surviving_entry_drains_on_subsequent_match() {
    // Edge case for Behavior 98: after the burnout consume drains the
    // filterless and burnout-filtered entries, the surviving 5.0
    // debt_collector-filtered entry drains on a subsequent matching
    // consume.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
    stack.add_one_shot_filtered(5.0, SourceId::from("protocol:debt_collector"));
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:debt_collector"))),
        5.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 99: one-shot — `add_one_shot_filtered` produces a filtered entry (verified via peek aggregate) ──

#[test]
fn one_shot_add_one_shot_filtered_writes_filter_some_observed_via_peek_views() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(4.0, SourceId::from("protocol:burnout"));
    // Non-matching peek returns 1.0.
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}

#[test]
fn one_shot_add_one_shot_filtered_three_peek_views_pin_filter_some() {
    // Edge case for Behavior 99: three peek calls — matching returns
    // 4.0, non-matching returns 1.0, None returns 1.0. All peeks; lane
    // is_empty remains false.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(4.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout"))),
        4.0,
    );
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 100: peek aggregate does NOT consume non-matching filtered entries ──

#[test]
fn peek_aggregate_does_not_consume_non_matching_filtered_entry() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}

#[test]
fn peek_non_matching_does_not_drain_lane_observable_via_consume() {
    // Edge case for Behavior 100: a follow-up matching consume returns
    // 2.0 — proves the prior peek didn't drain the lane.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_one_shots(Some(&SourceId::from("protocol:debt_collector")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

// ── Behavior 101: peek aggregate does NOT consume matching filtered entries either ──

#[test]
fn peek_aggregate_does_not_consume_matching_filtered_entry() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
    // Second peek of the same entry — still 2.0 (lane not drained).
    assert_f32_eq(
        stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn two_matching_peeks_followed_by_matching_consume_returns_full_value() {
    // Edge case for Behavior 101: a follow-up matching consume returns
    // 2.0 — pins that the two peeks did not consume.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout")));
    let _ = stack.aggregate_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

// ── Behavior 102: empty one-shot lane consume returns 1.0 for any emission ──

#[test]
fn empty_one_shot_lane_some_emission_consume_returns_one() {
    let mut stack = DamageBoostStack::default();
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(stack.is_empty());
}

#[test]
fn populated_then_drained_one_shot_lane_consume_again_returns_one() {
    // Edge case for Behavior 102: populate, drain via matching consume,
    // then call consume AGAIN on the now-empty lane: returns 1.0 and
    // is_empty is true.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(stack.is_empty());
}

// ── Behavior 103: `is_empty` is false after `add_filtered` ──

#[test]
fn is_empty_false_after_add_filtered() {
    let mut stack = DamageBoostStack::default();
    assert!(stack.is_empty());
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_reflects_presence_not_emission_match_for_filtered_persistent() {
    // Edge case for Behavior 103: aggregate_persistent(None) returns
    // 1.0 (filtered entry doesn't apply to None emission), but
    // is_empty() is still false — `is_empty` reflects entry presence.
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 104: `is_empty` is false after `add_one_shot_filtered` ──

#[test]
fn is_empty_false_after_add_one_shot_filtered() {
    let mut stack = DamageBoostStack::default();
    assert!(stack.is_empty());
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert!(!stack.is_empty());
}

#[test]
fn is_empty_reflects_presence_not_emission_match_for_filtered_one_shot() {
    // Edge case for Behavior 104: aggregate_one_shots(None) returns 1.0
    // (filtered entry doesn't apply to None peek), but is_empty() is
    // still false.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 105: `is_empty` is true again after a matching consume drains a filtered one-shot ──

#[test]
fn is_empty_true_after_matching_consume_drains_filtered_one_shot() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert!(stack.is_empty());
}

#[test]
fn second_matching_consume_after_drain_returns_one_and_is_empty() {
    // Edge case for Behavior 105: a second matching consume call
    // returns 1.0 and is_empty is still true.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot_filtered(2.0, SourceId::from("protocol:burnout"));
    let _ = stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout")));
    assert_f32_eq(
        stack.aggregate_and_consume_one_shots(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert!(stack.is_empty());
}
