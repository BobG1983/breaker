//! System to snapshot node highlights during chip selection.

use bevy::prelude::*;

use crate::state::run::{
    definition::HighlightConfig,
    resources::{HighlightTracker, RunState, RunStats},
    systems::select_highlights::select_highlights,
};

/// Drains `RunStats.highlights`, partitions by current node, selects the best
/// highlights from the current node via `select_highlights`, and rebuilds the
/// highlight list as previous-node entries + selected current-node entries.
/// Also clears per-node tracking counters in `HighlightTracker`.
pub(crate) fn snapshot_node_highlights(
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
    run_state: Res<RunState>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::resources::{HighlightKind, RunHighlight};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .add_systems(Update, snapshot_node_highlights);
        app
    }

    fn make_highlight(kind: HighlightKind, node_index: u32, value: f32) -> RunHighlight {
        RunHighlight {
            kind,
            node_index,
            value,
            detail: None,
        }
    }

    // -----------------------------------------------------------------------
    // Behavior 1: Partition-and-replace selects top highlights from current
    //             node, preserving previous-node entries
    // -----------------------------------------------------------------------

    #[test]
    fn partition_and_replace_preserves_previous_node_and_selects_current() {
        let mut app = test_app();

        // Set up 5 highlights: 2 from node 0, 3 from node 2
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::ClutchClear, 2, 1.5),
                make_highlight(HighlightKind::NoDamageNode, 2, 0.0),
                make_highlight(HighlightKind::PerfectStreak, 2, 7.0),
            ];
        }
        // highlight_cap defaults to 5, so all 3 node-2 entries fit
        app.world_mut().resource_mut::<RunState>().node_index = 2;

        // Dirty tracker to prove system runs
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_bolts_lost = 3;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            5,
            "total highlights should be 5: 2 previous-node + 3 selected current-node"
        );
        // The first 2 entries should be the node-0 entries (preserved)
        assert_eq!(
            stats.highlights[0].node_index, 0,
            "first entry should be from node 0"
        );
        assert_eq!(
            stats.highlights[1].node_index, 0,
            "second entry should be from node 0"
        );
        // The remaining entries should be from node 2
        for h in &stats.highlights[2..] {
            assert_eq!(
                h.node_index, 2,
                "entries after previous-node should be from current node 2"
            );
        }
        // Tracker per-node counters should be cleared
        assert_eq!(
            app.world().resource::<HighlightTracker>().node_bolts_lost,
            0,
            "per-node tracker fields should be cleared after snapshot"
        );
    }

    #[test]
    fn partition_and_replace_with_cap_1_keeps_single_best_current_node_entry() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::ClutchClear, 2, 1.5),
                make_highlight(HighlightKind::NoDamageNode, 2, 0.0),
                make_highlight(HighlightKind::PerfectStreak, 2, 7.0),
            ];
        }
        // Override highlight_cap to 1
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 1;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 2;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "total highlights should be 3: 2 previous-node + 1 selected current-node"
        );
        assert_eq!(stats.highlights[0].node_index, 0);
        assert_eq!(stats.highlights[1].node_index, 0);
        assert_eq!(stats.highlights[2].node_index, 2);
    }

    // -----------------------------------------------------------------------
    // Behavior 2: Current-node raw entries replaced by selected subset when
    //             cap is smaller than count
    // -----------------------------------------------------------------------

    #[test]
    fn current_node_entries_pruned_to_cap() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::ClutchClear, 1, 1.5),
                make_highlight(HighlightKind::NoDamageNode, 1, 0.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 1;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "total highlights should be 3: 2 previous-node + 1 best current-node"
        );
        // Verify previous-node entries preserved
        assert_eq!(stats.highlights[0].node_index, 0);
        assert_eq!(stats.highlights[1].node_index, 0);
        // Verify one current-node entry selected
        assert_eq!(stats.highlights[2].node_index, 1);
    }

    #[test]
    fn cap_zero_discards_all_current_node_entries() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::ClutchClear, 1, 1.5),
                make_highlight(HighlightKind::NoDamageNode, 1, 0.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 0;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            2,
            "cap=0 should discard all current-node entries, leaving only 2 previous-node"
        );
        assert_eq!(stats.highlights[0].node_index, 0);
        assert_eq!(stats.highlights[1].node_index, 0);
    }

    // -----------------------------------------------------------------------
    // Behavior 3: Snapshot clears per-node tracking counters in
    //             HighlightTracker after snapshotting
    // -----------------------------------------------------------------------

    #[test]
    fn clears_per_node_tracking_counters() {
        let mut app = test_app();

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 5;
            tracker.node_bolts_lost = 2;
            tracker.cell_destroyed_times = vec![1.0, 2.0, 3.0];
            tracker.non_perfect_bumps_this_node = 1;
            tracker.total_bumps_this_node = 6;
            tracker.cells_since_last_breaker_hit = 4;
            tracker.best_combo = 3;
            tracker.cell_bounces_since_breaker = 7;
            tracker.best_pinball_rally = 5;
        }

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "consecutive_perfect_bumps should reset to 0"
        );
        assert_eq!(
            tracker.node_bolts_lost, 0,
            "node_bolts_lost should reset to 0"
        );
        assert!(
            tracker.cell_destroyed_times.is_empty(),
            "cell_destroyed_times should be cleared"
        );
        assert_eq!(
            tracker.non_perfect_bumps_this_node, 0,
            "non_perfect_bumps_this_node should reset to 0"
        );
        assert_eq!(
            tracker.total_bumps_this_node, 0,
            "total_bumps_this_node should reset to 0"
        );
        assert_eq!(
            tracker.cells_since_last_breaker_hit, 0,
            "cells_since_last_breaker_hit should reset to 0"
        );
        assert_eq!(tracker.best_combo, 0, "best_combo should reset to 0");
        assert_eq!(
            tracker.cell_bounces_since_breaker, 0,
            "cell_bounces_since_breaker should reset to 0"
        );
        assert_eq!(
            tracker.best_pinball_rally, 0,
            "best_pinball_rally should reset to 0"
        );
    }

    #[test]
    fn preserves_cross_node_fields_while_clearing_per_node() {
        let mut app = test_app();

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            // Per-node fields (should be reset by the system)
            tracker.consecutive_perfect_bumps = 5;
            tracker.node_bolts_lost = 2;
            // Cross-node fields (must be preserved)
            tracker.best_perfect_streak = 10;
            tracker.consecutive_no_damage_nodes = 3;
            tracker.fastest_node_clear_secs = 4.2;
            tracker.first_evolution_recorded = true;
            tracker
                .evolution_damage
                .insert("Piercing Barrage".to_owned(), 250.0);
        }

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        // Per-node fields must be cleared (proves system ran)
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "consecutive_perfect_bumps should be reset to 0"
        );
        assert_eq!(
            tracker.node_bolts_lost, 0,
            "node_bolts_lost should be reset to 0"
        );
        // Cross-node fields must be preserved
        assert_eq!(
            tracker.best_perfect_streak, 10,
            "best_perfect_streak should be preserved"
        );
        assert_eq!(
            tracker.consecutive_no_damage_nodes, 3,
            "consecutive_no_damage_nodes should be preserved"
        );
        assert!(
            (tracker.fastest_node_clear_secs - 4.2).abs() < f32::EPSILON,
            "fastest_node_clear_secs should be preserved"
        );
        assert!(
            tracker.first_evolution_recorded,
            "first_evolution_recorded should be preserved"
        );
        assert_eq!(
            tracker.evolution_damage.get("Piercing Barrage"),
            Some(&250.0),
            "evolution_damage should be preserved"
        );
    }

    #[test]
    fn does_not_reset_node_start_time() {
        let mut app = test_app();

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.node_start_time = 15.0;
            // Dirty a per-node field to verify the system runs and clears it
            tracker.consecutive_perfect_bumps = 4;
        }

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            (tracker.node_start_time - 15.0).abs() < f32::EPSILON,
            "node_start_time should NOT be reset by snapshot_node_highlights (only by reset_highlight_tracker)"
        );
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "per-node fields should be cleared (proving the system ran)"
        );
    }

    #[test]
    fn clears_tracker_when_per_node_fields_already_default() {
        let mut app = test_app();

        // Add a highlight for current node to verify the system actively processes
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::NoDamageNode, 0, 0.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 1;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 0;

        // All per-node tracker fields are already at defaults (fresh tracker)
        // This should not panic or cause any mutation to the tracker
        app.update();

        // Verify the system ran (highlights were pruned to cap=1)
        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            1,
            "system should still run partition-and-replace even with default tracker"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(tracker.consecutive_perfect_bumps, 0);
        assert_eq!(tracker.node_bolts_lost, 0);
        assert!(tracker.cell_destroyed_times.is_empty());
        assert_eq!(tracker.non_perfect_bumps_this_node, 0);
        assert_eq!(tracker.total_bumps_this_node, 0);
        assert_eq!(tracker.cells_since_last_breaker_hit, 0);
        assert_eq!(tracker.best_combo, 0);
        assert_eq!(tracker.cell_bounces_since_breaker, 0);
        assert_eq!(tracker.best_pinball_rally, 0);
    }

    // -----------------------------------------------------------------------
    // Behavior 4: Snapshot does nothing to highlights when no entries exist
    //             for current node
    // -----------------------------------------------------------------------

    #[test]
    fn empty_highlights_remain_empty_after_snapshot() {
        let mut app = test_app();
        app.world_mut().resource_mut::<RunState>().node_index = 0;

        // Pre-set tracker counters to verify they are still cleared
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 3;
            tracker.node_bolts_lost = 1;
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "highlights should remain empty when no entries exist"
        );

        // Tracker counters should still be cleared
        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "tracker should still be cleared even with no highlights"
        );
        assert_eq!(
            tracker.node_bolts_lost, 0,
            "tracker should still be cleared even with no highlights"
        );
    }

    #[test]
    fn previous_node_highlights_preserved_when_no_current_node_entries() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::PerfectStreak, 0, 7.0),
            ];
        }
        // Current node is 1, but all highlights are from node 0
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 3;
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "all 3 previous-node highlights should be preserved"
        );
        for h in &stats.highlights {
            assert_eq!(
                h.node_index, 0,
                "all highlights should still be from node 0"
            );
        }

        // Tracker should still be cleared
        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(tracker.consecutive_perfect_bumps, 0);
    }

    // -----------------------------------------------------------------------
    // Behavior 5: Snapshot does nothing when no highlights match current
    //             node index
    // -----------------------------------------------------------------------

    #[test]
    fn no_current_node_highlights_preserves_all_previous() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
            ];
        }
        // Current node is 1, but highlights are only for node 0
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        // Dirty up tracker to verify it gets cleared
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 4;
            tracker.node_bolts_lost = 2;
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            2,
            "both node-0 entries should be preserved"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "per-node counters should still be cleared even with no current-node highlights"
        );
        assert_eq!(
            tracker.node_bolts_lost, 0,
            "node_bolts_lost should still be cleared even with no current-node highlights"
        );
    }

    #[test]
    fn handles_max_node_index_without_panic() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![make_highlight(HighlightKind::FastClear, 0, 0.0)];
        }
        app.world_mut().resource_mut::<RunState>().node_index = u32::MAX;

        // Dirty up tracker to verify the system runs
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 7;
        }

        // Should not panic
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            1,
            "node-0 highlight should be preserved with u32::MAX node_index"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "per-node counters should be cleared even with u32::MAX node_index"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 6: Snapshot is idempotent across multiple ChipSelect ticks
    // -----------------------------------------------------------------------

    #[test]
    fn idempotent_across_multiple_ticks() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::ClutchClear, 0, 2.0),
                make_highlight(HighlightKind::NoDamageNode, 0, 0.0),
            ];
        }
        app.world_mut().resource_mut::<RunState>().node_index = 0;

        // Dirty up tracker before first tick
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.consecutive_perfect_bumps = 3;
        }

        // First tick
        app.update();

        let len_after_first = app.world().resource::<RunStats>().highlights.len();
        assert_eq!(
            len_after_first, 2,
            "first tick: should have 2 selected entries (cap=5, both fit)"
        );
        // Tracker should be cleared after first tick
        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "first tick: tracker should be cleared"
        );

        // Second tick
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            2,
            "second tick: highlights should not duplicate, still 2 entries"
        );

        // Third tick for good measure
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            2,
            "third tick: highlights should remain stable at 2 entries"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 7: Snapshot filters highlights by current node_index only
    // -----------------------------------------------------------------------

    #[test]
    fn filters_by_current_node_index_only() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::ClutchClear, 1, 2.0),
                make_highlight(HighlightKind::NoDamageNode, 1, 0.0),
                make_highlight(HighlightKind::PerfectStreak, 2, 6.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 1;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "total: 1 node-0 + 1 node-2 (both previous) + 1 selected node-1 = 3"
        );

        // Count entries by node_index
        let node_0_count = stats
            .highlights
            .iter()
            .filter(|h| h.node_index == 0)
            .count();
        let node_1_count = stats
            .highlights
            .iter()
            .filter(|h| h.node_index == 1)
            .count();
        let node_2_count = stats
            .highlights
            .iter()
            .filter(|h| h.node_index == 2)
            .count();

        assert_eq!(node_0_count, 1, "node-0 entry should be preserved");
        assert_eq!(
            node_1_count, 1,
            "only 1 of the 2 node-1 entries should survive (cap=1)"
        );
        assert_eq!(
            node_2_count, 1,
            "future node-2 entry should be preserved as previous-node"
        );
    }

    #[test]
    fn future_node_highlight_treated_as_previous() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::FastClear, 0, 0.0),
                make_highlight(HighlightKind::PerfectStreak, 2, 6.0),
            ];
        }
        // Current node is 1, so node_index=2 is a "future" node
        app.world_mut().resource_mut::<RunState>().node_index = 1;

        // Dirty up tracker to verify the system runs
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.best_combo = 5;
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            2,
            "both entries are previous-node (neither is node_index=1), so both preserved"
        );
        assert_eq!(stats.highlights[0].node_index, 0);
        assert_eq!(stats.highlights[1].node_index, 2);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.best_combo, 0,
            "per-node counters should be cleared even when only previous-node highlights exist"
        );
    }

    // -----------------------------------------------------------------------
    // Behavior 8: Snapshot uses highlight_cap from HighlightConfig for
    //             selection count
    // -----------------------------------------------------------------------

    #[test]
    fn uses_highlight_cap_for_selection_count() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::ClutchClear, 0, 1.5),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::PerfectStreak, 0, 7.0),
                make_highlight(HighlightKind::NoDamageNode, 0, 0.0),
                make_highlight(HighlightKind::FastClear, 0, 0.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 3;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 0;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "with cap=3 and no previous-node entries, exactly 3 should survive"
        );
    }

    #[test]
    fn cap_zero_discards_all_current_node_entries_with_no_previous() {
        let mut app = test_app();

        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights = vec![
                make_highlight(HighlightKind::ClutchClear, 0, 1.5),
                make_highlight(HighlightKind::MassDestruction, 0, 12.0),
                make_highlight(HighlightKind::PerfectStreak, 0, 7.0),
                make_highlight(HighlightKind::NoDamageNode, 0, 0.0),
                make_highlight(HighlightKind::FastClear, 0, 0.0),
            ];
        }
        {
            let mut config = app.world_mut().resource_mut::<HighlightConfig>();
            config.highlight_cap = 0;
        }
        app.world_mut().resource_mut::<RunState>().node_index = 0;

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "cap=0 with no previous-node entries should result in empty highlights"
        );
    }
}
