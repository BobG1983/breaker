use crate::state::run::{
    resources::{HighlightKind, RunHighlight},
    run_end::systems::spawn_run_end_screen::tests::helpers::*,
};

/// Known highlight text prefixes used in `spawn_highlights_section`.
pub(super) fn is_highlight_text(text: &str) -> bool {
    text.starts_with("Clutch Clear")
        || text.starts_with("No Damage")
        || text.starts_with("Fast Clear")
        || text.starts_with("Perfect Streak")
        || text.starts_with("Mass Destruction")
        || text.starts_with("First Evolution")
        || text.starts_with("Most Powerful Evolution")
        || text.starts_with("Close Save")
        || text.starts_with("Speed Demon")
        || text.starts_with("Untouchable")
        || text.starts_with("Combo King")
        || text.starts_with("Pinball Wizard")
        || text.starts_with("Comeback")
        || text.starts_with("Perfect Node")
        || text.starts_with("Nail Biter")
}

pub(super) fn make_highlights(count: usize) -> Vec<RunHighlight> {
    let kinds = [
        HighlightKind::ClutchClear,
        HighlightKind::NoDamageNode,
        HighlightKind::FastClear,
        HighlightKind::PerfectStreak,
        HighlightKind::MassDestruction,
        HighlightKind::FirstEvolution,
    ];
    (0..count)
        .map(|i| RunHighlight {
            kind: kinds[i % kinds.len()].clone(),
            node_index: u32::try_from(i).unwrap_or(u32::MAX),
            value: 1.0,
            detail: None,
        })
        .collect()
}

/// Collects only highlight texts from all spawned Text entities, in spawn order.
pub(super) fn collect_highlight_texts(app: &mut bevy::prelude::App) -> Vec<String> {
    collect_texts(app)
        .into_iter()
        .filter(|t| is_highlight_text(t))
        .collect()
}
