//! Edge case tests for `score_highlight` — zero values, at-threshold, division guards.

use super::{
    super::system::score_highlight,
    helpers::{default_config, highlight},
};
use crate::prelude::*;

#[test]
fn score_lower_is_better_value_zero_clamped_to_max() {
    let config = default_config();
    // value=0.0 on lower-is-better: clamped to max(0.0, 0.1) = 0.1
    // raw = 3.0 / 0.1 = 30.0
    // normalized = clamp((30.0 - 1.0) / (10.0 - 1.0), 0.0, 1.0) = clamp(29.0/9.0, ..) = 1.0
    let h = highlight(HighlightKind::ClutchClear, 0.0);
    let (score, _) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "ClutchClear(0.0) should clamp to 1.0 after dividing by 0.1, got {score}"
    );
}

#[test]
fn score_at_threshold_returns_normalized_zero() {
    let config = default_config();
    // PerfectStreak with value == threshold: raw = 5.0 / 5.0 = 1.0
    // normalized = (1.0 - 1.0) / (4.0 - 1.0) = 0.0
    let h = highlight(HighlightKind::PerfectStreak, 5.0);
    let (score, _) = score_highlight(&h, &config);
    assert!(
        score.abs() < 0.01,
        "PerfectStreak at exact threshold should score ~0.0, got {score}"
    );
}

#[test]
fn score_clutch_clear_max_expected_1_0_no_division_by_zero() {
    // max_expected_clutch_clear = 1.0 triggers the (max_expected - 1.0) = 0.0 denominator guard
    // Should return normalized = 1.0 instead of dividing by zero
    let mut config = default_config();
    config.max_expected_clutch_clear = 1.0;

    // ClutchClear is lower-is-better: raw = threshold / value = 3.0 / 1.0 = 3.0
    // Normalized denominator = (1.0 - 1.0) = 0.0 -> guard returns 1.0
    let h = highlight(HighlightKind::ClutchClear, 1.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "ClutchClear with max_expected=1.0 should return 1.0 (division guard), got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}

#[test]
fn score_lower_is_better_at_threshold_returns_normalized_zero() {
    let config = default_config();
    // ClutchClear with value == threshold: raw = 3.0 / 3.0 = 1.0
    // normalized = (1.0 - 1.0) / (10.0 - 1.0) = 0.0
    let h = highlight(HighlightKind::ClutchClear, 3.0);
    let (score, _) = score_highlight(&h, &config);
    assert!(
        score.abs() < 0.01,
        "ClutchClear at exact threshold should score ~0.0, got {score}"
    );
}
