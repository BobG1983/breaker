//! Tests for `score_highlight` on higher-is-better highlight kinds.

use super::{
    super::system::score_highlight,
    helpers::{default_config, highlight},
};
use crate::state::run::resources::{HighlightCategory, HighlightKind};

#[test]
fn score_perfect_streak_value_10_threshold_5_normalized_0_333() {
    let config = default_config();
    // perfect_streak_count=5, max_expected_perfect_streak=4.0
    // raw = value / threshold = 10.0 / 5.0 = 2.0
    // normalized = clamp((2.0 - 1.0) / (4.0 - 1.0), 0.0, 1.0) = 1.0/3.0 = 0.333
    let h = highlight(HighlightKind::PerfectStreak, 10.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0 / 3.0).abs() < 0.01,
        "PerfectStreak(10.0) should score ~0.333, got {score}"
    );
    assert_eq!(category, HighlightCategory::Execution);
}

#[test]
fn score_mass_destruction_value_20_threshold_10_normalized_0_25() {
    let config = default_config();
    // mass_destruction_count=10, max_expected_mass_destruction=5.0
    // raw = 20.0 / 10.0 = 2.0
    // normalized = clamp((2.0 - 1.0) / (5.0 - 1.0), 0.0, 1.0) = 1.0/4.0 = 0.25
    let h = highlight(HighlightKind::MassDestruction, 20.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 0.25).abs() < 0.01,
        "MassDestruction(20.0) should score ~0.25, got {score}"
    );
    assert_eq!(category, HighlightCategory::Execution);
}

#[test]
fn score_combo_king_value_16_threshold_8_normalized_0_333() {
    let config = default_config();
    // combo_king_cells=8, max_expected_combo_king=4.0
    // raw = 16.0 / 8.0 = 2.0
    // normalized = (2.0 - 1.0) / (4.0 - 1.0) = 0.333
    let h = highlight(HighlightKind::ComboKing, 16.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0 / 3.0).abs() < 0.01,
        "ComboKing(16.0) should score ~0.333, got {score}"
    );
    assert_eq!(category, HighlightCategory::Execution);
}

#[test]
fn score_pinball_wizard_value_24_threshold_12_normalized_0_333() {
    let config = default_config();
    // pinball_wizard_bounces=12, max_expected_pinball_wizard=4.0
    // raw = 24.0 / 12.0 = 2.0
    // normalized = (2.0 - 1.0) / (4.0 - 1.0) = 0.333
    let h = highlight(HighlightKind::PinballWizard, 24.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0 / 3.0).abs() < 0.01,
        "PinballWizard(24.0) should score ~0.333, got {score}"
    );
    assert_eq!(category, HighlightCategory::Execution);
}

#[test]
fn score_untouchable_value_4_threshold_2_normalized_0_25() {
    let config = default_config();
    // untouchable_nodes=2, max_expected_untouchable=5.0
    // raw = 4.0 / 2.0 = 2.0
    // normalized = (2.0 - 1.0) / (5.0 - 1.0) = 0.25
    let h = highlight(HighlightKind::Untouchable, 4.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 0.25).abs() < 0.01,
        "Untouchable(4.0) should score ~0.25, got {score}"
    );
    assert_eq!(category, HighlightCategory::Endurance);
}

#[test]
fn score_comeback_value_5_threshold_3_normalized_0_333() {
    let config = default_config();
    // comeback_bolts_lost=3, max_expected_comeback=3.0
    // raw = 5.0 / 3.0 = 1.667
    // normalized = (1.667 - 1.0) / (3.0 - 1.0) = 0.667/2.0 = 0.333
    let h = highlight(HighlightKind::Comeback, 5.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0 / 3.0).abs() < 0.01,
        "Comeback(5.0) should score ~0.333, got {score}"
    );
    assert_eq!(category, HighlightCategory::Endurance);
}

#[test]
fn score_perfect_node_value_10_threshold_1_normalized_0_474() {
    let config = default_config();
    // PerfectNode: threshold is fixed at 1.0, max_expected_perfect_node=20.0
    // raw = 10.0 / 1.0 = 10.0
    // normalized = (10.0 - 1.0) / (20.0 - 1.0) = 9.0/19.0 = 0.4737
    let h = highlight(HighlightKind::PerfectNode, 10.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 9.0 / 19.0).abs() < 0.01,
        "PerfectNode(10.0) should score ~0.474, got {score}"
    );
    assert_eq!(category, HighlightCategory::Execution);
}
