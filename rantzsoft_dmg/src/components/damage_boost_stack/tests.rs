use bevy::prelude::*;

use super::component::DamageBoostStack;
use crate::SourceId;

/// Compares two `f32` values for "equality" without tripping
/// `clippy::float_cmp`. Handles infinity by checking sign and finiteness
/// directly (so we never compute `INF - INF`, which produces NaN).
#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

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

// ── Behavior 80: `aggregate_one_shots` returns 1.0 when empty ──

#[test]
fn aggregate_one_shots_empty_returns_one() {
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

#[test]
fn aggregate_one_shots_on_default_stack_does_not_panic_and_is_idempotent() {
    // Edge case for Behavior 80: a default-constructed stack must NOT
    // panic when called. Two consecutive calls both return 1.0
    // (idempotent on empty input).
    let stack = DamageBoostStack::default();
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

// ── Behavior 81: single one-shot returns the value ──

#[test]
fn aggregate_one_shots_single_value_returns_that_value() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.5);
    assert_f32_eq(stack.aggregate_one_shots(None), 2.5);
}

#[test]
fn aggregate_one_shots_single_identity_value_returns_one_but_is_not_empty() {
    // Edge case for Behavior 81: a one-shot of identity 1.0 returns 1.0
    // from `aggregate_one_shots()` — same value as the empty case, but
    // `is_empty()` is false. Pins the lane semantic.
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(1.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
    assert!(!stack.is_empty());
}

// ── Behavior 82: multiple one-shots return the product ──

#[test]
fn aggregate_one_shots_multiple_values_returns_product() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(1.5);
    stack.add_one_shot(3.0);
    // 2.0 * 1.5 * 3.0 = 9.0.
    assert_f32_eq(stack.aggregate_one_shots(None), 9.0);
}

#[test]
fn aggregate_one_shots_is_order_independent() {
    // Edge case for Behavior 82: order independence. Build a second
    // stack with the same three values appended in reverse order.
    // Pin BOTH stacks to the concrete value 9.0 independently —
    // do NOT compare forward vs reverse (would pass a constant-RED stub).
    let mut forward = DamageBoostStack::default();
    forward.add_one_shot(2.0);
    forward.add_one_shot(1.5);
    forward.add_one_shot(3.0);

    let mut reverse = DamageBoostStack::default();
    reverse.add_one_shot(3.0);
    reverse.add_one_shot(1.5);
    reverse.add_one_shot(2.0);

    assert_f32_eq(forward.aggregate_one_shots(None), 9.0);
    assert_f32_eq(reverse.aggregate_one_shots(None), 9.0);
}

// ── Behavior 83: peek does not consume — multiple calls return same value ──

#[test]
fn aggregate_one_shots_does_not_consume_across_repeated_calls() {
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    // Both peek calls return 6.0.
    assert_f32_eq(stack.aggregate_one_shots(None), 6.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 6.0);
    // A subsequent consume ALSO returns 6.0 — observable proof the peek
    // calls did NOT drain the lane.
    assert_f32_eq(stack.aggregate_and_consume_one_shots(None), 6.0);
}

#[test]
fn aggregate_one_shots_after_consume_returns_one() {
    // Edge case for Behavior 83: after the consume, a follow-up peek
    // returns 1.0 (lane is now empty — confirms consume drained AND
    // that the new peek method correctly observes the post-drain
    // empty state).
    let mut stack = DamageBoostStack::default();
    stack.add_one_shot(2.0);
    stack.add_one_shot(3.0);
    let _ = stack.aggregate_and_consume_one_shots(None);
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

// ── Behavior 84: `aggregate_one_shots` ignores the persistent lane ──

#[test]
fn aggregate_one_shots_ignores_persistent_lane() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    // Persistent only — one-shot lane is untouched.
    assert_f32_eq(stack.aggregate_one_shots(None), 1.0);
}

#[test]
fn aggregate_one_shots_isolated_from_persistent_when_both_populated() {
    // Edge case for Behavior 84: persistent + one-shot — peek of one_shots
    // returns only the one-shot product (2.0); persistent lane still
    // returns 5.0 (peek did not corrupt the persistent lane).
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 5.0);
    stack.add_one_shot(2.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 2.0);
    assert_f32_eq(stack.aggregate_persistent(None), 5.0);
}

// ── Behavior 85: symmetric counterpart to `aggregate_persistent` does not pollute it ──

#[test]
fn aggregate_one_shots_and_persistent_are_each_isolated() {
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    // Call peek of one_shots, then persistent — neither lane is mutated.
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
    assert_f32_eq(stack.aggregate_persistent(None), 7.0);
}

#[test]
fn aggregate_persistent_then_one_shots_are_each_isolated() {
    // Edge case for Behavior 85: reverse the call order.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    assert_f32_eq(stack.aggregate_persistent(None), 7.0);
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
}

#[test]
fn aggregate_one_shots_idempotent_after_cross_method_calls() {
    // Edge case for Behavior 85: call peek of one_shots again after
    // both calls — still 11.0. Pins idempotence under cross-method
    // calls.
    let mut stack = DamageBoostStack::default();
    stack.add(SourceId::from("src:alpha"), 7.0);
    stack.add_one_shot(11.0);
    let _ = stack.aggregate_one_shots(None);
    let _ = stack.aggregate_persistent(None);
    assert_f32_eq(stack.aggregate_one_shots(None), 11.0);
}

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
