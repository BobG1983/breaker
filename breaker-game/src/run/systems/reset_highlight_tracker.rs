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
}
