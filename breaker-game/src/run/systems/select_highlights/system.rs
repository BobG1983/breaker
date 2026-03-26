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
