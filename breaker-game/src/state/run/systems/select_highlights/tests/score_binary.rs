//! Tests for `score_highlight` on binary highlight kinds.

use super::{
    super::system::score_highlight,
    helpers::{default_config, highlight},
};
use crate::prelude::*;

#[test]
fn score_no_damage_node_binary_returns_1_0() {
    let config = default_config();
    // Binary type: raw=1.0, normalized=1.0
    let h = highlight(HighlightKind::NoDamageNode, 0.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "NoDamageNode should score 1.0 (binary), got {score}"
    );
    assert_eq!(category, HighlightCategory::Endurance);
}

#[test]
fn score_fast_clear_binary_returns_1_0() {
    let config = default_config();
    let h = highlight(HighlightKind::FastClear, 0.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "FastClear should score 1.0 (binary), got {score}"
    );
    assert_eq!(category, HighlightCategory::Clutch);
}

#[test]
fn score_first_evolution_binary_returns_1_0() {
    let config = default_config();
    let h = highlight(HighlightKind::FirstEvolution, 1.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "FirstEvolution should score 1.0 (binary), got {score}"
    );
    assert_eq!(category, HighlightCategory::Progression);
}

#[test]
fn score_most_powerful_evolution_binary_returns_1_0() {
    let config = default_config();
    // MostPowerfulEvolution is binary (currently unreachable but should score correctly)
    let h = highlight(HighlightKind::MostPowerfulEvolution, 1.0);
    let (score, category) = score_highlight(&h, &config);
    assert!(
        (score - 1.0).abs() < f32::EPSILON,
        "MostPowerfulEvolution should score 1.0 (binary), got {score}"
    );
    assert_eq!(category, HighlightCategory::Progression);
}
