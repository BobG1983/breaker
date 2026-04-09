//! System to detect `ComboKing` highlights from cell destruction streaks between breaker contacts.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltImpactBreaker,
    cells::messages::CellDestroyedAt,
    state::run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
};

/// Reads [`CellDestroyedAt`] and [`BoltImpactBreaker`] messages
/// to detect `ComboKing` highlights.
///
/// - `CellDestroyedAt` increments `cells_since_last_breaker_hit`.
/// - `BoltImpactBreaker` checks the combo threshold, records the highlight, and resets the counter.
pub(crate) fn detect_combo_king(
    mut cell_destroyed_reader: MessageReader<CellDestroyedAt>,
    mut bolt_hit_breaker_reader: MessageReader<BoltImpactBreaker>,
    config: Res<HighlightConfig>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<NodeOutcome>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    // Increment cells destroyed since last breaker hit
    for _msg in cell_destroyed_reader.read() {
        tracker.cells_since_last_breaker_hit += 1;
    }

    // On breaker hit: check threshold, record highlight, reset counter
    for _msg in bolt_hit_breaker_reader.read() {
        let node_index = run_state.node_index;

        tracker.best_combo = tracker.best_combo.max(tracker.cells_since_last_breaker_hit);

        if tracker.cells_since_last_breaker_hit >= config.combo_king_cells {
            // Always emit for juice
            writer.write(HighlightTriggered {
                kind: HighlightKind::ComboKing,
            });

            // Record in stats — dedup by kind
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::ComboKing);
            if !already {
                let count = tracker.cells_since_last_breaker_hit;
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::ComboKing,
                    node_index,
                    value: f32::from(u16::try_from(count).unwrap_or(u16::MAX)),
                    detail: None,
                });
            }
        }

        tracker.cells_since_last_breaker_hit = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::messages::CellDestroyedAt,
        state::run::resources::{HighlightKind, RunHighlight},
    };

    // --- TestMessages resources for each message type ---

    #[derive(Resource, Default)]
    struct TestCellDestroyed(Vec<CellDestroyedAt>);

    #[derive(Resource, Default)]
    struct TestBoltImpactBreaker(Vec<BoltImpactBreaker>);

    fn enqueue_cell_destroyed(
        msg_res: Res<TestCellDestroyed>,
        mut writer: MessageWriter<CellDestroyedAt>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn enqueue_bolt_hit_breaker(
        msg_res: Res<TestBoltImpactBreaker>,
        mut writer: MessageWriter<BoltImpactBreaker>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct CapturedHighlightTriggered(Vec<HighlightTriggered>);

    fn collect_highlight_triggered(
        mut reader: MessageReader<HighlightTriggered>,
        mut captured: ResMut<CapturedHighlightTriggered>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .with_message::<CellDestroyedAt>()
            .with_message::<BoltImpactBreaker>()
            .with_message::<HighlightTriggered>()
            .with_resource::<RunStats>()
            .with_resource::<HighlightTracker>()
            .with_resource::<NodeOutcome>()
            .insert_resource(HighlightConfig::default())
            .with_resource::<TestCellDestroyed>()
            .with_resource::<TestBoltImpactBreaker>()
            .with_resource::<CapturedHighlightTriggered>()
            .with_system(
                FixedUpdate,
                (
                    (enqueue_cell_destroyed, enqueue_bolt_hit_breaker),
                    detect_combo_king,
                    collect_highlight_triggered,
                )
                    .chain(),
            )
            .build()
    }

    use crate::shared::test_utils::tick;

    // --- Behavior 6: CellDestroyedAt increments cells_since_last_breaker_hit ---

    #[test]
    fn cell_destroyed_increments_cells_since_last_breaker_hit() {
        let mut app = test_app();
        app.insert_resource(TestCellDestroyed(vec![
            CellDestroyedAt {
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                was_required_to_clear: false,
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cells_since_last_breaker_hit, 3,
            "3 CellDestroyedAt messages should set counter to 3"
        );
    }

    // --- Behavior 7: ComboKing detected when counter >= 8 ---

    #[test]
    fn combo_king_detected_when_counter_reaches_threshold() {
        let mut app = test_app();
        // Default combo_king_cells = 8
        // Pre-set counter to 8 (at threshold), then send BoltImpactBreaker
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cells_since_last_breaker_hit = 8;
        app.insert_resource(TestBoltImpactBreaker(vec![BoltImpactBreaker {
            bolt: Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let combo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::ComboKing);
        assert!(
            combo.is_some(),
            "should detect ComboKing when cells_since_last_breaker_hit=8 >= combo_king_cells=8"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cells_since_last_breaker_hit, 0,
            "counter should reset to 0 after BoltImpactBreaker"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::ComboKing);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered with ComboKing kind"
        );
    }

    // --- Behavior 8: ComboKing NOT detected when counter < 8 ---

    #[test]
    fn combo_king_not_detected_when_counter_below_threshold() {
        let mut app = test_app();
        // Pre-set counter to 7 (below threshold of 8)
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cells_since_last_breaker_hit = 7;
        app.insert_resource(TestBoltImpactBreaker(vec![BoltImpactBreaker {
            bolt: Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let combo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::ComboKing);
        assert!(
            combo.is_none(),
            "should NOT detect ComboKing when cells=7 < threshold=8"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cells_since_last_breaker_hit, 0,
            "counter should still reset to 0 after BoltImpactBreaker"
        );
        assert_eq!(
            tracker.best_combo, 7,
            "best_combo should be updated to 7 even when below threshold"
        );
    }

    // --- Behavior 9: best_combo tracks maximum ---

    #[test]
    fn best_combo_tracks_maximum_across_resets() {
        let mut app = test_app();
        // Pre-set best_combo to 10, current counter to 5
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.best_combo = 10;
            tracker.cells_since_last_breaker_hit = 5;
        }
        app.insert_resource(TestBoltImpactBreaker(vec![BoltImpactBreaker {
            bolt: Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.best_combo, 10,
            "best_combo should remain 10 when current combo (5) is lower"
        );
    }

    // --- Behavior 14: Dedup — only one ComboKing in RunStats ---

    #[test]
    fn dedup_only_one_combo_king_in_run_stats() {
        let mut app = test_app();
        // Pre-fill with existing ComboKing
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::ComboKing,
                node_index: 0,
                value: 10.0,
                detail: None,
            });
        }

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.cells_since_last_breaker_hit = 10;
        }
        app.insert_resource(TestBoltImpactBreaker(vec![BoltImpactBreaker {
            bolt: Entity::PLACEHOLDER,
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let combo_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::ComboKing)
            .count();
        assert_eq!(
            combo_count, 1,
            "should NOT add a second ComboKing highlight"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg_count = captured
            .0
            .iter()
            .filter(|h| h.kind == HighlightKind::ComboKing)
            .count();
        assert_eq!(
            msg_count, 1,
            "should still emit HighlightTriggered even when deduped in RunStats"
        );
    }
}
