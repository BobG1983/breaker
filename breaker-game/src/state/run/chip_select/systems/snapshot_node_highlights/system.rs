//! System to snapshot node highlights during chip selection.

use bevy::prelude::*;

use crate::{
    prelude::*,
    state::run::{
        definition::HighlightConfig,
        resources::{HighlightTracker, NodeOutcome},
        systems::select_highlights::select_highlights,
    },
};

/// Drains `RunStats.highlights`, partitions by current node, selects the best
/// highlights from the current node via `select_highlights`, and rebuilds the
/// highlight list as previous-node entries + selected current-node entries.
/// Also clears per-node tracking counters in `HighlightTracker`.
pub(crate) fn snapshot_node_highlights(
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
    run_state: Res<NodeOutcome>,
    config: Res<HighlightConfig>,
) {
    // Step 1: Drain all highlights and partition by node index.
    let current_node = run_state.node_index;
    let (current, previous): (Vec<_>, Vec<_>) = stats
        .highlights
        .drain(..)
        .partition(|h| h.node_index == current_node);

    // Step 2: Select best highlights from the current node.
    let selected_indices = select_highlights(&current, &config, config.highlight_cap as usize);

    // Step 3: Rebuild highlights vec — previous-node entries + selected current-node entries.
    stats.highlights = previous;
    for &i in &selected_indices {
        stats.highlights.push(current[i].clone());
    }

    // Step 4: Reset per-node tracking counters in HighlightTracker.
    tracker.consecutive_perfect_bumps = 0;
    tracker.node_bolts_lost = 0;
    tracker.cell_destroyed_times.clear();
    tracker.non_perfect_bumps_this_node = 0;
    tracker.total_bumps_this_node = 0;
    tracker.cells_since_last_breaker_hit = 0;
    tracker.best_combo = 0;
    tracker.cell_bounces_since_breaker = 0;
    tracker.best_pinball_rally = 0;
}
