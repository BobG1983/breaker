//! System to reset per-node highlight tracking state between nodes.

use bevy::prelude::*;

use crate::run::resources::HighlightTracker;

/// Resets per-node fields in [`HighlightTracker`] when entering a new node.
///
/// Preserves `best_perfect_streak` across resets.
pub(crate) fn reset_highlight_tracker(
    mut tracker: ResMut<HighlightTracker>,
    time: Res<Time<Fixed>>,
) {
    tracker.node_bolts_lost = 0;
    tracker.consecutive_perfect_bumps = 0;
    tracker.cell_destroyed_times.clear();
    tracker.node_start_time = time.elapsed_secs();
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HighlightTracker>()
            .add_systems(Update, reset_highlight_tracker);
        app
    }

    #[test]
    fn resets_per_node_fields() {
        let mut app = test_app();
        // Set up dirty tracker state
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.node_bolts_lost = 5;
        tracker.consecutive_perfect_bumps = 3;
        tracker.cell_destroyed_times = vec![1.0, 2.0];
        tracker.node_start_time = 10.0;

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.node_bolts_lost, 0,
            "node_bolts_lost should reset to 0"
        );
        assert_eq!(
            tracker.consecutive_perfect_bumps, 0,
            "consecutive_perfect_bumps should reset to 0"
        );
        assert!(
            tracker.cell_destroyed_times.is_empty(),
            "cell_destroyed_times should be cleared"
        );
    }

    #[test]
    fn preserves_best_perfect_streak_across_resets() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .best_perfect_streak = 8;

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.best_perfect_streak, 8,
            "best_perfect_streak should NOT be reset"
        );
    }

    // --- Behavior 22: resets new per-node fields, preserves all cross-node fields ---

    #[test]
    fn resets_new_per_node_fields_preserves_cross_node_fields() {
        let mut app = test_app();
        // Set up dirty per-node AND cross-node state
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        // Per-node fields (should be reset)
        tracker.non_perfect_bumps_this_node = 3;
        tracker.total_bumps_this_node = 7;
        tracker.cells_since_last_breaker_hit = 5;
        tracker.best_combo = 4;
        tracker.cell_bounces_since_breaker = 6;
        tracker.best_pinball_rally = 3;
        // Cross-node fields (should be preserved)
        tracker.consecutive_no_damage_nodes = 2;
        tracker.fastest_node_clear_secs = 4.5;
        tracker.first_evolution_recorded = true;
        tracker.best_perfect_streak = 10;

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        // Per-node fields should be reset
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
        // Cross-node fields should be preserved
        assert_eq!(
            tracker.consecutive_no_damage_nodes, 2,
            "consecutive_no_damage_nodes should NOT be reset"
        );
        assert!(
            (tracker.fastest_node_clear_secs - 4.5).abs() < f32::EPSILON,
            "fastest_node_clear_secs should NOT be reset"
        );
        assert!(
            tracker.first_evolution_recorded,
            "first_evolution_recorded should NOT be reset"
        );
        assert_eq!(
            tracker.best_perfect_streak, 10,
            "best_perfect_streak should NOT be reset"
        );
    }
}
