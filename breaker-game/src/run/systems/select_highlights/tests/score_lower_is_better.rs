//! Tests for `score_highlight` on lower-is-better highlight kinds.

use super::{
    super::system::score_highlight,
    helpers::{default_config, highlight},
};
use crate::run::resources::{HighlightCategory, HighlightKind};

#[test]
fn score_clutch_clear_value_1_threshold_3_normalized_0_222() {
    let config = default_config();
    // clutch_clear_secs=3.0, max_expected_clutch_clear=10.0
    // raw = threshold / value = 3.0 / 1.0 = 3.0
    // normalized = (3.0 - 1.0) / (10.0 - 1.0) = 2.0/9.0 = 0.222
    let h = highlight(HighlightKind::ClutchClear, 1.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 2.0 / 9.0).abs() < 0.01,
        "ClutchClear(1.0) should score ~0.222, got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}

#[test]
fn score_speed_demon_value_2_threshold_5_normalized_0_167() {
    let config = default_config();
    // speed_demon_secs=5.0, max_expected_speed_demon=10.0
    // raw = 5.0 / 2.0 = 2.5
    // normalized = (2.5 - 1.0) / (10.0 - 1.0) = 1.5/9.0 = 0.167
    let h = highlight(HighlightKind::SpeedDemon, 2.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.5 / 9.0).abs() < 0.01,
        "SpeedDemon(2.0) should score ~0.167, got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}

#[test]
fn score_close_save_value_5_threshold_20_normalized_0_333() {
    let config = default_config();
    // close_save_pixels=20.0, max_expected_close_save=10.0
    // raw = 20.0 / 5.0 = 4.0
    // normalized = (4.0 - 1.0) / (10.0 - 1.0) = 3.0/9.0 = 0.333
    let h = highlight(HighlightKind::CloseSave, 5.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 3.0 / 9.0).abs() < 0.01,
        "CloseSave(5.0) should score ~0.333, got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}

#[test]
fn score_nail_biter_value_10_threshold_30_normalized_0_222() {
    let config = default_config();
    // nail_biter_pixels=30.0, max_expected_nail_biter=10.0
    // raw = 30.0 / 10.0 = 3.0
    // normalized = (3.0 - 1.0) / (10.0 - 1.0) = 2.0/9.0 = 0.222
    let h = highlight(HighlightKind::NailBiter, 10.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 2.0 / 9.0).abs() < 0.01,
        "NailBiter(10.0) should score ~0.222, got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}
