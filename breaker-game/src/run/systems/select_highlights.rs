//! Highlight scoring and diversity-penalized selection.
//!
//! Pure functions (not Bevy systems) for scoring highlights and selecting
//! the most impressive/diverse subset for run-end display.

use std::collections::HashMap;

use crate::run::{
    definition::HighlightConfig,
    resources::{HighlightCategory, HighlightKind, RunHighlight},
};

/// Lossless u32→f32 for small config values (bounded by u16).
fn config_f32(val: u32) -> f32 {
    f32::from(u16::try_from(val).unwrap_or(u16::MAX))
}

/// Score a highlight and return its normalized score (0.0–1.0) and category.
pub(crate) fn score_highlight(
    highlight: &RunHighlight,
    config: &HighlightConfig,
) -> (f32, HighlightCategory) {
    let category = highlight.kind.category();

    // Binary types always score 1.0
    match highlight.kind {
        HighlightKind::NoDamageNode
        | HighlightKind::FastClear
        | HighlightKind::FirstEvolution
        | HighlightKind::MostPowerfulEvolution => return (1.0, category),
        _ => {}
    }

    // Compute raw score based on type
    let (raw, max_expected) = match highlight.kind {
        // Higher-is-better: raw = value / threshold
        HighlightKind::PerfectStreak => (
            highlight.value / config_f32(config.perfect_streak_count),
            config.max_expected_perfect_streak,
        ),
        HighlightKind::MassDestruction => (
            highlight.value / config_f32(config.mass_destruction_count),
            config.max_expected_mass_destruction,
        ),
        HighlightKind::ComboKing => (
            highlight.value / config_f32(config.combo_king_cells),
            config.max_expected_combo_king,
        ),
        HighlightKind::PinballWizard => (
            highlight.value / config_f32(config.pinball_wizard_bounces),
            config.max_expected_pinball_wizard,
        ),
        HighlightKind::Untouchable => (
            highlight.value / config_f32(config.untouchable_nodes),
            config.max_expected_untouchable,
        ),
        HighlightKind::Comeback => (
            highlight.value / config_f32(config.comeback_bolts_lost),
            config.max_expected_comeback,
        ),
        // Higher-is-better raw value: threshold is 1.0
        HighlightKind::PerfectNode => (highlight.value, config.max_expected_perfect_node),
        // Lower-is-better: raw = threshold / max(value, 0.1)
        HighlightKind::ClutchClear => (
            config.clutch_clear_secs / highlight.value.max(0.1),
            config.max_expected_clutch_clear,
        ),
        HighlightKind::SpeedDemon => (
            config.speed_demon_secs / highlight.value.max(0.1),
            config.max_expected_speed_demon,
        ),
        HighlightKind::CloseSave => (
            config.close_save_pixels / highlight.value.max(0.1),
            config.max_expected_close_save,
        ),
        HighlightKind::NailBiter => (
            config.nail_biter_pixels / highlight.value.max(0.1),
            config.max_expected_nail_biter,
        ),
        // Binary types handled above
        _ => unreachable!(),
    };

    // Normalize
    if max_expected <= 1.0 {
        return (1.0, category);
    }
    let normalized = ((raw - 1.0) / (max_expected - 1.0)).clamp(0.0, 1.0);
    (normalized, category)
}

/// Select the top highlights using diversity-penalized greedy selection.
///
/// Returns indices into the `highlights` slice in selection order.
pub(crate) fn select_highlights(
    highlights: &[RunHighlight],
    config: &HighlightConfig,
    count: usize,
) -> Vec<usize> {
    if highlights.is_empty() {
        return Vec::new();
    }

    // Score all highlights
    let scored: Vec<(usize, f32, HighlightCategory)> = highlights
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let (score, category) = score_highlight(h, config);
            (i, score, category)
        })
        .collect();

    let mut category_picks: HashMap<HighlightCategory, u32> = HashMap::new();
    let mut result = Vec::with_capacity(count.min(highlights.len()));
    let mut remaining: Vec<usize> = (0..scored.len()).collect();

    for _ in 0..count {
        if remaining.is_empty() {
            break;
        }

        // Find candidate with max adjusted score; on tie, prefer lower original index
        let mut best_idx_in_remaining = 0;
        let mut best_adjusted = f32::NEG_INFINITY;
        let mut best_original_idx = usize::MAX;

        for (ri, &candidate) in remaining.iter().enumerate() {
            let (original_idx, normalized, category) = scored[candidate];
            let picks = category_picks.get(&category).copied().unwrap_or(0);
            let adjusted = normalized * config.diversity_penalty.powi(picks.cast_signed());

            if adjusted > best_adjusted
                || ((adjusted - best_adjusted).abs() < f32::EPSILON
                    && original_idx < best_original_idx)
            {
                best_adjusted = adjusted;
                best_idx_in_remaining = ri;
                best_original_idx = original_idx;
            }
        }

        let chosen = remaining.remove(best_idx_in_remaining);
        let (original_idx, _, category) = scored[chosen];
        result.push(original_idx);
        *category_picks.entry(category).or_insert(0) += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{HighlightCategory, HighlightKind, RunHighlight};

    fn default_config() -> HighlightConfig {
        HighlightConfig::default()
    }

    fn highlight(kind: HighlightKind, value: f32) -> RunHighlight {
        RunHighlight {
            kind,
            node_index: 0,
            value,
            detail: None,
        }
    }

    // ========================================================================
    // Part B: score_highlight — higher-is-better types
    // ========================================================================

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

    // ========================================================================
    // Part B: score_highlight — lower-is-better types
    // ========================================================================

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

    // ========================================================================
    // Part B: score_highlight — binary types
    // ========================================================================

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

    // ========================================================================
    // Part B: score_highlight — edge cases
    // ========================================================================

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
        // Normalized denominator = (1.0 - 1.0) = 0.0 → guard returns 1.0
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

    // ========================================================================
    // Part C: select_highlights — diversity-penalized selection
    // ========================================================================

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
            // PerfectStreak: value=10, threshold=5, max=4 → normalized ~0.333
            highlight(HighlightKind::PerfectStreak, 10.0),
            // ComboKing: value=16, threshold=8, max=4 → normalized ~0.333
            highlight(HighlightKind::ComboKing, 16.0),
            // PinballWizard: value=24, threshold=12, max=4 → normalized ~0.333
            highlight(HighlightKind::PinballWizard, 24.0),
            // MassDestruction: value=20, threshold=10, max=5 → normalized ~0.25
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
            // PerfectNode: value=10, threshold=1.0, max=20 → normalized ~0.474
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
            // ClutchClear: value=0.5, threshold=3.0 → raw=6.0 → normalized=(6-1)/(10-1)=0.556 (Clutch)
            highlight(HighlightKind::ClutchClear, 0.5),
            // SpeedDemon: value=1.0, threshold=5.0 → raw=5.0 → normalized=(5-1)/(10-1)=0.444 (Clutch)
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
}
