//! `preview_damage` — pure free function that previews the final damage
//! value for a given base, an optional dealer-side `DamageBoostStack`, and
//! an optional target-side `VulnerableStack`. Both stacks are taken by
//! shared reference, so the function CANNOT consume the one-shot lanes
//! at the type level. The pipeline (`apply_damage_boosts::<T>`,
//! `apply_vulnerable::<T>`) remains the sole consumer of one-shots.
//!
//! Semantic contract: **one-shots are COUNTED but NEVER consumed.**

use crate::{DamageBoostStack, VulnerableStack};

/// Preview the final damage value for `base` after applying every
/// multiplicative lane on the dealer's `DamageBoostStack` AND the
/// target's `VulnerableStack`. Both stacks are optional; missing stacks
/// contribute the multiplicative identity `1.0`.
///
/// Both lanes (persistent AND one-shots) are counted on each stack.
/// Neither lane is consumed — `&` borrows make consumption impossible at
/// the type level.
#[must_use]
pub fn preview_damage(
    base: f32,
    boosts: Option<&DamageBoostStack>,
    vuln: Option<&VulnerableStack>,
) -> f32 {
    let boost_mult = boosts.map_or(1.0, |s| s.aggregate_persistent() * s.aggregate_one_shots());
    let vuln_mult = vuln.map_or(1.0, |s| s.aggregate_persistent() * s.aggregate_one_shots());
    base * boost_mult * vuln_mult
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DamageBoostStack, SourceId, VulnerableStack};

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

    // ── Behavior 92: both stacks `None` returns base unchanged ──

    #[test]
    fn both_none_returns_base_unchanged() {
        assert_f32_eq(preview_damage(10.0, None, None), 10.0);
    }

    #[test]
    fn both_none_with_zero_base_returns_zero() {
        // Edge case for Behavior 92: base = 0.0 → returns 0.0.
        assert_f32_eq(preview_damage(0.0, None, None), 0.0);
    }

    #[test]
    fn both_none_with_negative_base_preserves_sign_unchanged() {
        // Edge case for Behavior 92: base = -3.5 → returns -3.5 (no sign
        // flip, no clamping).
        assert_f32_eq(preview_damage(-3.5, None, None), -3.5);
    }

    // ── Behavior 93: boost persistent lane only ──

    #[test]
    fn boost_persistent_lane_only_multiplies_base() {
        let mut boost = DamageBoostStack::default();
        boost.add(SourceId::from("src:alpha"), 2.0);
        // 10.0 * 2.0 * 1.0 (empty one-shot lane) * 1.0 (no vuln) = 20.0.
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
    }

    #[test]
    fn boost_persistent_lane_only_is_idempotent_and_does_not_mutate() {
        // Edge case for Behavior 93: a SECOND identical call also returns
        // 20.0 AND `boost.aggregate_persistent()` still returns 2.0
        // afterwards (peek did not mutate the persistent lane).
        let mut boost = DamageBoostStack::default();
        boost.add(SourceId::from("src:alpha"), 2.0);
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
        assert_f32_eq(boost.aggregate_persistent(), 2.0);
    }

    // ── Behavior 94: boost one-shot lane only ──

    #[test]
    fn boost_one_shot_lane_only_multiplies_base() {
        let mut boost = DamageBoostStack::default();
        boost.add_one_shot(2.0);
        // 10.0 * 1.0 (empty persistent) * 2.0 (one-shot) * 1.0 (no vuln) = 20.0.
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
    }

    #[test]
    fn boost_one_shot_lane_is_not_consumed_across_repeated_previews() {
        // Edge case for Behavior 94: a SECOND identical call also returns
        // 20.0 AND a subsequent `aggregate_and_consume_one_shots()` still
        // returns 2.0 — observable proof that NEITHER preview call drained
        // the one-shot lane.
        let mut boost = DamageBoostStack::default();
        boost.add_one_shot(2.0);
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 20.0);
        assert_f32_eq(boost.aggregate_and_consume_one_shots(), 2.0);
    }

    // ── Behavior 95: vuln persistent lane only ──

    #[test]
    fn vuln_persistent_lane_only_multiplies_base() {
        let mut vuln = VulnerableStack::default();
        vuln.add(SourceId::from("mark:fragility"), 3.0);
        // 10.0 * 1.0 (no boost) * 3.0 = 30.0.
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
    }

    #[test]
    fn vuln_persistent_lane_only_is_idempotent_and_does_not_mutate() {
        // Edge case for Behavior 95: a SECOND identical call also returns
        // 30.0 AND `vuln.aggregate_persistent()` still returns 3.0
        // afterwards.
        let mut vuln = VulnerableStack::default();
        vuln.add(SourceId::from("mark:fragility"), 3.0);
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
        assert_f32_eq(vuln.aggregate_persistent(), 3.0);
    }

    // ── Behavior 96: vuln one-shot lane only ──

    #[test]
    fn vuln_one_shot_lane_only_multiplies_base() {
        let mut vuln = VulnerableStack::default();
        vuln.add_one_shot(3.0);
        // 10.0 * 1.0 (no boost) * 1.0 (empty persistent) * 3.0 (one-shot) = 30.0.
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
    }

    #[test]
    fn vuln_one_shot_lane_is_not_consumed_across_repeated_previews() {
        // Edge case for Behavior 96: a SECOND identical call also returns
        // 30.0 AND a subsequent `aggregate_and_consume_one_shots()` returns
        // 3.0 — observable proof of non-consumption across BOTH preview
        // calls.
        let mut vuln = VulnerableStack::default();
        vuln.add_one_shot(3.0);
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 30.0);
        assert_f32_eq(vuln.aggregate_and_consume_one_shots(), 3.0);
    }

    // ── Behavior 97: full chain — both stacks, both lanes populated ──

    #[test]
    fn full_chain_both_stacks_both_lanes_returns_literal_product() {
        let mut boost = DamageBoostStack::default();
        boost.add(SourceId::from("src:alpha"), 2.0);
        boost.add_one_shot(1.5);

        let mut vuln = VulnerableStack::default();
        vuln.add(SourceId::from("mark:fragility"), 3.0);
        vuln.add_one_shot(1.25);

        // 10.0 * (2.0 * 1.5) * (3.0 * 1.25) = 10.0 * 3.0 * 3.75 = 112.5.
        // Use literal 112.5 — do NOT compute at runtime (would mask formula
        // bugs).
        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 112.5);
    }

    // ── Behavior 98: idempotent on one-shots — neither stack mutated by repeated previews ──

    #[test]
    fn three_repeated_previews_do_not_consume_one_shots() {
        let mut boost = DamageBoostStack::default();
        boost.add(SourceId::from("src:alpha"), 2.0);
        boost.add_one_shot(1.5);

        let mut vuln = VulnerableStack::default();
        vuln.add(SourceId::from("mark:fragility"), 3.0);
        vuln.add_one_shot(1.25);

        // Three previews — all return 112.5.
        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 112.5);
        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 112.5);
        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 112.5);

        // After the three previews, consume returns the original one-shot
        // values — proves they were never drained.
        assert_f32_eq(boost.aggregate_and_consume_one_shots(), 1.5);
        assert_f32_eq(vuln.aggregate_and_consume_one_shots(), 1.25);

        // Persistent lanes are untouched by either peek or consume.
        assert_f32_eq(boost.aggregate_persistent(), 2.0);
        assert_f32_eq(vuln.aggregate_persistent(), 3.0);
    }

    #[test]
    fn fourth_preview_after_consume_returns_persistent_only_product() {
        // Edge case for Behavior 98: a FOURTH preview call AFTER the
        // consumes returns 60.0 (= 10.0 * 2.0 * 3.0 — the one-shot lanes
        // are now empty and contribute identity 1.0 each; persistents
        // still contribute 2.0 and 3.0).
        let mut boost = DamageBoostStack::default();
        boost.add(SourceId::from("src:alpha"), 2.0);
        boost.add_one_shot(1.5);

        let mut vuln = VulnerableStack::default();
        vuln.add(SourceId::from("mark:fragility"), 3.0);
        vuln.add_one_shot(1.25);

        let _ = preview_damage(10.0, Some(&boost), Some(&vuln));
        let _ = preview_damage(10.0, Some(&boost), Some(&vuln));
        let _ = preview_damage(10.0, Some(&boost), Some(&vuln));

        let _ = boost.aggregate_and_consume_one_shots();
        let _ = vuln.aggregate_and_consume_one_shots();

        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 60.0);
    }

    // ── Behavior 99: empty `Some(&...)` stacks behave like `None` ──

    #[test]
    fn empty_some_stacks_behave_like_none() {
        let boost = DamageBoostStack::default();
        let vuln = VulnerableStack::default();
        assert_f32_eq(preview_damage(10.0, Some(&boost), Some(&vuln)), 10.0);
    }

    #[test]
    fn empty_some_boost_with_none_vuln_returns_base() {
        // Edge case for Behavior 99: mix one empty-`Some` with one `None`.
        let boost = DamageBoostStack::default();
        assert_f32_eq(preview_damage(10.0, Some(&boost), None), 10.0);
    }

    #[test]
    fn none_boost_with_empty_some_vuln_returns_base() {
        // Edge case for Behavior 99: mirror of the above — both branches
        // reach the 1.0 * 1.0 terminal state through different code paths.
        let vuln = VulnerableStack::default();
        assert_f32_eq(preview_damage(10.0, None, Some(&vuln)), 10.0);
    }
}
