//! Tests for `select_highlights` — diversity-penalized selection algorithm.

use super::{
    super::system::select_highlights,
    helpers::{default_config, highlight},
};
use crate::state::run::resources::{HighlightKind, RunHighlight};

#[test]
fn select_highlights_different_categories_picks_top_by_score() {
    let config = default_config();
    // 5 highlights from different categories, cap=3
    // Use binary highlights for known score (1.0) and varying scored ones
    let highlights = vec![
        // score 1.0 (binary) — Endurance
        highlight(HighlightKind::NoDamageNode, 0.0),
        // score 1.0 (binary) — Clutch
        highlight(HighlightKind::FastClear, 0.0),
        // score ~0.333 — Execution (PerfectStreak, value=10, threshold=5, max=4)
        highlight(HighlightKind::PerfectStreak, 10.0),
        // score 1.0 (binary) — Progression
        highlight(HighlightKind::FirstEvolution, 1.0),
        // score ~0.25 — Execution (MassDestruction, value=20, threshold=10, max=5)
        highlight(HighlightKind::MassDestruction, 20.0),
    ];

    let result = select_highlights(&highlights, &config, 3);
    assert_eq!(result.len(), 3, "should select exactly 3 highlights");
    // The three 1.0-scored items (indices 0, 1, 3) should be selected.
    // On tie, prefer lower index.
    assert_eq!(
        result[0], 0,
        "first pick should be index 0 (NoDamageNode, score=1.0, lowest index among ties)"
    );
    assert_eq!(
        result[1], 1,
        "second pick should be index 1 (FastClear, score=1.0)"
    );
    assert_eq!(
        result[2], 3,
        "third pick should be index 3 (FirstEvolution, score=1.0)"
    );
}

#[test]
fn select_highlights_same_category_applies_diversity_penalty() {
    // 4 highlights all Execution category
    // default diversity_penalty = 0.5
    let config = default_config();

    // All Execution: PerfectStreak, ComboKing, PinballWizard, MassDestruction
    let highlights = vec![
        // PerfectStreak: value=10, threshold=5, max=4 -> normalized ~0.333
        highlight(HighlightKind::PerfectStreak, 10.0),
        // ComboKing: value=16, threshold=8, max=4 -> normalized ~0.333
        highlight(HighlightKind::ComboKing, 16.0),
        // PinballWizard: value=24, threshold=12, max=4 -> normalized ~0.333
        highlight(HighlightKind::PinballWizard, 24.0),
        // MassDestruction: value=20, threshold=10, max=5 -> normalized ~0.25
        highlight(HighlightKind::MassDestruction, 20.0),
    ];

    let result = select_highlights(&highlights, &config, 4);
    assert_eq!(result.len(), 4, "should select all 4 highlights");

    // First pick: index 0 (score ~0.333, lowest index among ~0.333 ties)
    assert_eq!(
        result[0], 0,
        "first pick should be index 0 (PerfectStreak, highest score, lowest index on tie)"
    );
    // After first pick, all remaining get penalty 0.5:
    // idx 1: 0.333 * 0.5 = 0.167
    // idx 2: 0.333 * 0.5 = 0.167
    // idx 3: 0.25 * 0.5 = 0.125
    // Second pick: index 1 (0.167, lowest index on tie with idx 2)
    assert_eq!(
        result[1], 1,
        "second pick should be index 1 (ComboKing, adjusted 0.167, lowest index on tie)"
    );
    // After second pick, remaining get another penalty 0.5:
    // idx 2: 0.333 * 0.5^2 = 0.0833
    // idx 3: 0.25 * 0.5^2 = 0.0625
    assert_eq!(
        result[2], 2,
        "third pick should be index 2 (PinballWizard, adjusted 0.0833)"
    );
    assert_eq!(
        result[3], 3,
        "fourth pick should be index 3 (MassDestruction, adjusted 0.0625)"
    );
}

#[test]
fn select_highlights_fewer_than_cap_returns_all() {
    let config = default_config();
    let highlights = vec![
        highlight(HighlightKind::NoDamageNode, 0.0),
        highlight(HighlightKind::FastClear, 0.0),
    ];

    let result = select_highlights(&highlights, &config, 5);
    assert_eq!(
        result.len(),
        2,
        "should return all 2 highlights when cap is 5"
    );
}

#[test]
fn select_highlights_empty_returns_empty() {
    let config = default_config();
    let highlights: Vec<RunHighlight> = vec![];

    let result = select_highlights(&highlights, &config, 3);
    assert!(
        result.is_empty(),
        "should return empty vec for empty highlights"
    );
}

#[test]
fn select_highlights_tie_breaking_prefers_lower_index() {
    let config = default_config();
    // Two binary highlights with identical score=1.0 from different categories
    let highlights = vec![
        highlight(HighlightKind::FastClear, 0.0),      // Clutch, 1.0
        highlight(HighlightKind::NoDamageNode, 0.0),   // Endurance, 1.0
        highlight(HighlightKind::FirstEvolution, 1.0), // Progression, 1.0
    ];

    let result = select_highlights(&highlights, &config, 1);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0], 0,
        "on tie, should prefer lower index (index 0 over 1 and 2)"
    );
}

#[test]
fn select_highlights_no_penalty_when_diversity_penalty_is_1_0() {
    // diversity_penalty=1.0 means multiply by 1.0 — no penalty at all.
    // So selection should be pure score order.
    let mut config = default_config();
    config.diversity_penalty = 1.0;

    // All Execution category
    let highlights = vec![
        // PerfectStreak: score ~0.333
        highlight(HighlightKind::PerfectStreak, 10.0),
        // MassDestruction: score ~0.25
        highlight(HighlightKind::MassDestruction, 20.0),
        // PerfectNode: value=10, threshold=1.0, max=20 -> normalized ~0.474
        highlight(HighlightKind::PerfectNode, 10.0),
    ];

    let result = select_highlights(&highlights, &config, 3);
    assert_eq!(result.len(), 3);
    // With no penalty, pure score order:
    // idx 2 (PerfectNode ~0.474) > idx 0 (PerfectStreak ~0.333) > idx 1 (MassDestruction ~0.25)
    assert_eq!(
        result[0], 2,
        "with no penalty, first pick should be highest score (PerfectNode ~0.474)"
    );
    assert_eq!(result[1], 0, "second pick should be PerfectStreak (~0.333)");
    assert_eq!(result[2], 1, "third pick should be MassDestruction (~0.25)");
}

#[test]
fn select_highlights_mixed_categories_diversity_favors_variety() {
    let config = default_config();
    // Mix of categories: one high-scoring Endurance, two lower-scoring Clutch
    let highlights = vec![
        // NoDamageNode: binary 1.0 (Endurance)
        highlight(HighlightKind::NoDamageNode, 0.0),
        // ClutchClear: value=0.5, threshold=3.0 -> raw=6.0 -> normalized=(6-1)/(10-1)=0.556 (Clutch)
        highlight(HighlightKind::ClutchClear, 0.5),
        // SpeedDemon: value=1.0, threshold=5.0 -> raw=5.0 -> normalized=(5-1)/(10-1)=0.444 (Clutch)
        highlight(HighlightKind::SpeedDemon, 1.0),
    ];

    let result = select_highlights(&highlights, &config, 3);
    assert_eq!(result.len(), 3);
    // First pick: idx 0 (NoDamageNode, score=1.0)
    assert_eq!(result[0], 0, "first pick: NoDamageNode (1.0)");
    // No penalty on Clutch yet, so:
    // idx 1 (ClutchClear ~0.556), idx 2 (SpeedDemon ~0.444)
    assert_eq!(result[1], 1, "second pick: ClutchClear (~0.556)");
    // After picking idx 1 (Clutch), idx 2 adjusted = 0.444 * 0.5 = 0.222
    assert_eq!(result[2], 2, "third pick: SpeedDemon (penalized ~0.222)");
}
