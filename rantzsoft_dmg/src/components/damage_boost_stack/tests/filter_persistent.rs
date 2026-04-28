use super::{super::component::DamageBoostStack, helpers::assert_f32_eq};
use crate::SourceId;

// ── Behavior 86: persistent — filterless entry × matched emission applies ──

#[test]
fn persistent_filterless_entry_applies_to_matched_emission() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn persistent_filterless_entry_applies_to_none_emission() {
    // Edge case for Behavior 86: filterless entries are unconditional —
    // also apply to a `None` emission.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    assert_f32_eq(stack.aggregate_persistent(None), 2.0);
}

// ── Behavior 87: persistent — filtered entry × matched emission applies ──

#[test]
fn persistent_filtered_entry_applies_to_matched_emission() {
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

#[test]
fn persistent_filtered_entry_match_works_across_cow_variants() {
    // Edge case for Behavior 87: emission built from String (Cow::Owned)
    // matches filter built from &'static str (Cow::Borrowed).
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    let emission = SourceId::from(String::from("protocol:burnout"));
    assert_f32_eq(stack.aggregate_persistent(Some(&emission)), 2.0);
}

// ── Behavior 88: persistent — filtered entry × wrong-source emission does NOT apply ──

#[test]
fn persistent_filtered_entry_does_not_apply_to_wrong_source_emission() {
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}

#[test]
fn persistent_non_matching_aggregate_does_not_mutate_lane() {
    // Edge case for Behavior 88: after a non-matching aggregate call,
    // a follow-up matching aggregate still returns 2.0 — proves the
    // entry was preserved.
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    let _ = stack.aggregate_persistent(Some(&SourceId::from("protocol:debt_collector")));
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        2.0,
    );
}

// ── Behavior 89: persistent — filtered entry × None emission does NOT apply ──

#[test]
fn persistent_filtered_entry_does_not_apply_to_none_emission() {
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

#[test]
fn persistent_filtered_entry_survives_none_emission_aggregate() {
    // Edge case for Behavior 89: after the None-emission aggregate, the
    // entry is still present (is_empty is false).
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        2.0,
        SourceId::from("protocol:burnout"),
    );
    let _ = stack.aggregate_persistent(None);
    assert!(!stack.is_empty());
}

// ── Behavior 90: persistent — mixed lane multiplies only matching entries ──

#[test]
fn persistent_mixed_lane_burnout_emission_multiplies_filterless_and_burnout_filtered() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add_filtered(
        SourceId::from("src:beta"),
        3.0,
        SourceId::from("protocol:burnout"),
    );
    stack.add_filtered(
        SourceId::from("src:gamma"),
        5.0,
        SourceId::from("protocol:debt_collector"),
    );
    // 2.0 (filterless) * 3.0 (burnout-filtered) = 6.0; gamma's 5.0 is
    // filtered to debt_collector and excluded.
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        6.0,
    );
}

#[test]
fn persistent_mixed_lane_three_emission_views_yield_three_distinct_products() {
    // Edge case for Behavior 90: three independent aggregate calls on
    // the same fully-populated stack yield three distinct concrete
    // products. Pins that aggregation is read-only and emission-driven.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add_filtered(
        SourceId::from("src:beta"),
        3.0,
        SourceId::from("protocol:burnout"),
    );
    stack.add_filtered(
        SourceId::from("src:gamma"),
        5.0,
        SourceId::from("protocol:debt_collector"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:debt_collector"))),
        10.0, // 2.0 * 5.0
    );
    assert_f32_eq(stack.aggregate_persistent(None), 2.0);
    // Re-check burnout view to confirm none of the prior calls drained.
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        6.0,
    );
}

// ── Behavior 91: persistent — `add_filtered` produces a filtered entry (verified via aggregate) ──

#[test]
fn persistent_add_filtered_writes_filter_some_observed_via_aggregate_views() {
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        4.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        4.0,
    );
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}

#[test]
fn persistent_add_filtered_does_not_apply_to_none_emission() {
    // Edge case for Behavior 91: the same entry returns 1.0 for `None`
    // emission. Pins that `add_filtered` writes `filter: Some(_)`, not
    // `filter: None`.
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        4.0,
        SourceId::from("protocol:burnout"),
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
}

// ── Behavior 92: persistent — `remove_by_source` is filter-blind ──

#[test]
fn persistent_remove_by_source_removes_filtered_entries_too() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 2.0);
    stack.add_filtered(
        SourceId::from("src:alpha"),
        3.0,
        SourceId::from("protocol:burnout"),
    );
    stack.remove_by_source(&SourceId::from("src:alpha"));
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
    assert_f32_eq(stack.aggregate_persistent(None), 1.0);
    assert!(stack.is_empty());
}

#[test]
fn persistent_remove_by_source_filter_blind_pins_no_filter_narrowing() {
    // Edge case for Behavior 92: pins that `remove_by_source` matches
    // entries by `source` field only — prevents a regression where a
    // future implementer narrows removal to only filterless entries.
    // Distinct sources are NOT collateral-removed.
    let mut stack = DamageBoostStack::default();
    stack.add_filtered(
        SourceId::from("src:alpha"),
        7.0,
        SourceId::from("protocol:burnout"),
    );
    stack.add(SourceId::from("src:beta"), 11.0);
    stack.remove_by_source(&SourceId::from("src:alpha"));
    // Only beta's filterless 11.0 survives.
    assert_f32_eq(stack.aggregate_persistent(None), 11.0);
}

// ── Behavior 93: persistent — empty-stack aggregate returns 1.0 for any emission ──

#[test]
fn persistent_aggregate_empty_with_some_emission_returns_one() {
    let stack = DamageBoostStack::default();
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:burnout"))),
        1.0,
    );
}

#[test]
fn persistent_aggregate_empty_with_other_some_emission_returns_one() {
    // Edge case for Behavior 93: empty-stack identity holds for ANY
    // emission, not just the legacy `None` case.
    let stack = DamageBoostStack::default();
    assert_f32_eq(
        stack.aggregate_persistent(Some(&SourceId::from("protocol:debt_collector"))),
        1.0,
    );
}
