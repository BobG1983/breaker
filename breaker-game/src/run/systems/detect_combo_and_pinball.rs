//! System to detect `ComboKing` and `PinballWizard` highlights from cell destruction
//! and cell bounce streaks between breaker contacts.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell},
    cells::messages::CellDestroyed,
    run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
};

/// Bundled collision message readers for combo/pinball detection.
#[derive(SystemParam)]
pub(crate) struct ComboMessages<'w, 's> {
    cell_destroyed: MessageReader<'w, 's, CellDestroyed>,
    bolt_hit_cell: MessageReader<'w, 's, BoltHitCell>,
    bolt_hit_breaker: MessageReader<'w, 's, BoltHitBreaker>,
}

/// Reads [`CellDestroyed`], [`BoltHitCell`], and [`BoltHitBreaker`] messages to
/// detect `ComboKing` and `PinballWizard` highlights.
///
/// - `CellDestroyed` increments `cells_since_last_breaker_hit`.
/// - `BoltHitCell` increments `cell_bounces_since_breaker`.
/// - `BoltHitBreaker` checks thresholds, records highlights, and resets counters.
pub(crate) fn detect_combo_and_pinball(
    mut messages: ComboMessages,
    config: Res<HighlightConfig>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    // Increment cells destroyed since last breaker hit
    for _msg in messages.cell_destroyed.read() {
        tracker.cells_since_last_breaker_hit += 1;
    }

    // Increment cell bounces since breaker contact
    for _msg in messages.bolt_hit_cell.read() {
        tracker.cell_bounces_since_breaker += 1;
    }

    // On breaker hit: check thresholds, record highlights, reset counters
    for _msg in messages.bolt_hit_breaker.read() {
        let node_index = run_state.node_index;

        // --- ComboKing ---
        tracker.best_combo = tracker.best_combo.max(tracker.cells_since_last_breaker_hit);

        if tracker.cells_since_last_breaker_hit >= config.combo_king_cells {
            // Always emit for juice
            writer.write(HighlightTriggered {
                kind: HighlightKind::ComboKing,
            });

            // Only record once in stats
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::ComboKing);
            if !already && stats.highlights.len() < config.highlight_cap as usize {
                let count = tracker.cells_since_last_breaker_hit;
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::ComboKing,
                    node_index,
                    value: f32::from(u16::try_from(count).unwrap_or(u16::MAX)),
                });
            }
        }

        tracker.cells_since_last_breaker_hit = 0;

        // --- PinballWizard ---
        tracker.best_pinball_rally = tracker
            .best_pinball_rally
            .max(tracker.cell_bounces_since_breaker);

        if tracker.cell_bounces_since_breaker >= config.pinball_wizard_bounces {
            // Always emit for juice
            writer.write(HighlightTriggered {
                kind: HighlightKind::PinballWizard,
            });

            // Only record once in stats
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::PinballWizard);
            if !already && stats.highlights.len() < config.highlight_cap as usize {
                let count = tracker.cell_bounces_since_breaker;
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::PinballWizard,
                    node_index,
                    value: f32::from(u16::try_from(count).unwrap_or(u16::MAX)),
                });
            }
        }

        tracker.cell_bounces_since_breaker = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{HighlightKind, RunHighlight};

    // --- TestMessages resources for each message type ---

    #[derive(Resource, Default)]
    struct TestCellDestroyed(Vec<CellDestroyed>);

    #[derive(Resource, Default)]
    struct TestBoltHitCell(Vec<BoltHitCell>);

    #[derive(Resource, Default)]
    struct TestBoltHitBreaker(Vec<BoltHitBreaker>);

    fn enqueue_cell_destroyed(
        msg_res: Res<TestCellDestroyed>,
        mut writer: MessageWriter<CellDestroyed>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn enqueue_bolt_hit_cell(
        msg_res: Res<TestBoltHitCell>,
        mut writer: MessageWriter<BoltHitCell>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn enqueue_bolt_hit_breaker(
        msg_res: Res<TestBoltHitBreaker>,
        mut writer: MessageWriter<BoltHitBreaker>,
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
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .add_message::<BoltHitCell>()
            .add_message::<BoltHitBreaker>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .init_resource::<TestCellDestroyed>()
            .init_resource::<TestBoltHitCell>()
            .init_resource::<TestBoltHitBreaker>()
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                FixedUpdate,
                (
                    (
                        enqueue_cell_destroyed,
                        enqueue_bolt_hit_cell,
                        enqueue_bolt_hit_breaker,
                    ),
                    detect_combo_and_pinball,
                    collect_highlight_triggered,
                )
                    .chain(),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Behavior 6: CellDestroyed increments cells_since_last_breaker_hit ---

    #[test]
    fn cell_destroyed_increments_cells_since_last_breaker_hit() {
        let mut app = test_app();
        app.insert_resource(TestCellDestroyed(vec![
            CellDestroyed {
                was_required_to_clear: true,
            },
            CellDestroyed {
                was_required_to_clear: true,
            },
            CellDestroyed {
                was_required_to_clear: false,
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cells_since_last_breaker_hit, 3,
            "3 CellDestroyed messages should set counter to 3"
        );
    }

    // --- Behavior 7: ComboKing detected when counter >= 8 ---

    #[test]
    fn combo_king_detected_when_counter_reaches_threshold() {
        let mut app = test_app();
        // Default combo_king_cells = 8
        // Pre-set counter to 8 (at threshold), then send BoltHitBreaker
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cells_since_last_breaker_hit = 8;
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
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
            "counter should reset to 0 after BoltHitBreaker"
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
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
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
            "counter should still reset to 0 after BoltHitBreaker"
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
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.best_combo, 10,
            "best_combo should remain 10 when current combo (5) is lower"
        );
    }

    // --- Behavior 10: BoltHitCell increments cell_bounces_since_breaker ---

    #[test]
    fn bolt_hit_cell_increments_cell_bounces_since_breaker() {
        let mut app = test_app();
        app.insert_resource(TestBoltHitCell(vec![
            BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt: Entity::PLACEHOLDER,
            },
            BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt: Entity::PLACEHOLDER,
            },
            BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt: Entity::PLACEHOLDER,
            },
            BoltHitCell {
                cell: Entity::PLACEHOLDER,
                bolt: Entity::PLACEHOLDER,
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cell_bounces_since_breaker, 4,
            "4 BoltHitCell messages should set counter to 4"
        );
    }

    // --- Behavior 11: PinballWizard detected when bounces >= 12 ---

    #[test]
    fn pinball_wizard_detected_when_bounces_reach_threshold() {
        let mut app = test_app();
        // Default pinball_wizard_bounces = 12
        // Pre-set counter to 12 (at threshold), then send BoltHitBreaker
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cell_bounces_since_breaker = 12;
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let pinball = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PinballWizard);
        assert!(
            pinball.is_some(),
            "should detect PinballWizard when cell_bounces=12 >= pinball_wizard_bounces=12"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::PinballWizard);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered with PinballWizard kind"
        );
    }

    // --- Behavior 12: PinballWizard NOT detected when bounces < 12 ---

    #[test]
    fn pinball_wizard_not_detected_when_bounces_below_threshold() {
        let mut app = test_app();
        // Pre-set counter to 11 (below threshold of 12)
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cell_bounces_since_breaker = 11;
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let pinball = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PinballWizard);
        assert!(
            pinball.is_none(),
            "should NOT detect PinballWizard when bounces=11 < threshold=12"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cell_bounces_since_breaker, 0,
            "counter should reset to 0 after BoltHitBreaker"
        );
        assert_eq!(
            tracker.best_pinball_rally, 11,
            "best_pinball_rally should be updated to 11 even when below threshold"
        );
    }

    // --- Behavior 13: Both ComboKing + PinballWizard can fire on same BoltHitBreaker ---

    #[test]
    fn both_combo_king_and_pinball_wizard_fire_on_same_breaker_hit() {
        let mut app = test_app();
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.cells_since_last_breaker_hit = 10; // >= 8
            tracker.cell_bounces_since_breaker = 15; // >= 12
        }
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let combo = stats
            .highlights
            .iter()
            .any(|h| h.kind == HighlightKind::ComboKing);
        let pinball = stats
            .highlights
            .iter()
            .any(|h| h.kind == HighlightKind::PinballWizard);
        assert!(combo, "should detect ComboKing");
        assert!(pinball, "should detect PinballWizard");

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let combo_msg = captured
            .0
            .iter()
            .any(|h| h.kind == HighlightKind::ComboKing);
        let pinball_msg = captured
            .0
            .iter()
            .any(|h| h.kind == HighlightKind::PinballWizard);
        assert!(combo_msg, "should emit HighlightTriggered for ComboKing");
        assert!(
            pinball_msg,
            "should emit HighlightTriggered for PinballWizard"
        );
    }

    // --- Behavior 14: Dedup — only one of each kind in RunStats ---

    #[test]
    fn dedup_only_one_of_each_kind_in_run_stats() {
        let mut app = test_app();
        // Pre-fill with existing ComboKing and PinballWizard
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::ComboKing,
                node_index: 0,
                value: 10.0,
            });
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::PinballWizard,
                node_index: 0,
                value: 15.0,
            });
        }

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.cells_since_last_breaker_hit = 10;
            tracker.cell_bounces_since_breaker = 15;
        }
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let combo_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::ComboKing)
            .count();
        let pinball_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::PinballWizard)
            .count();
        assert_eq!(
            combo_count, 1,
            "should NOT add a second ComboKing highlight"
        );
        assert_eq!(
            pinball_count, 1,
            "should NOT add a second PinballWizard highlight"
        );
    }
}
