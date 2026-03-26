//! System to detect `PinballWizard` highlights from cell bounce streaks between breaker contacts.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell},
    run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
};

/// Reads [`BoltHitCell`] and [`BoltHitBreaker`] messages
/// to detect `PinballWizard` highlights.
///
/// - `BoltHitCell` increments `cell_bounces_since_breaker`.
/// - `BoltHitBreaker` checks the pinball threshold, records the highlight, and resets the counter.
pub(crate) fn detect_pinball_wizard(
    mut bolt_hit_cell_reader: MessageReader<BoltHitCell>,
    mut bolt_hit_breaker_reader: MessageReader<BoltHitBreaker>,
    config: Res<HighlightConfig>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    // Increment cell bounces since breaker contact
    for _msg in bolt_hit_cell_reader.read() {
        tracker.cell_bounces_since_breaker += 1;
    }

    // On breaker hit: check threshold, record highlight, reset counter
    for _msg in bolt_hit_breaker_reader.read() {
        let node_index = run_state.node_index;

        tracker.best_pinball_rally = tracker
            .best_pinball_rally
            .max(tracker.cell_bounces_since_breaker);

        if tracker.cell_bounces_since_breaker >= config.pinball_wizard_bounces {
            // Always emit for juice
            writer.write(HighlightTriggered {
                kind: HighlightKind::PinballWizard,
            });

            // Record in stats — dedup by kind
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::PinballWizard);
            if !already {
                let count = tracker.cell_bounces_since_breaker;
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::PinballWizard,
                    node_index,
                    value: f32::from(u16::try_from(count).unwrap_or(u16::MAX)),
                    detail: None,
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
    struct TestBoltHitCell(Vec<BoltHitCell>);

    #[derive(Resource, Default)]
    struct TestBoltHitBreaker(Vec<BoltHitBreaker>);

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
            .add_message::<BoltHitCell>()
            .add_message::<BoltHitBreaker>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .init_resource::<TestBoltHitCell>()
            .init_resource::<TestBoltHitBreaker>()
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                FixedUpdate,
                (
                    (enqueue_bolt_hit_cell, enqueue_bolt_hit_breaker),
                    detect_pinball_wizard,
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

    // --- Behavior 14: Dedup — only one PinballWizard in RunStats ---

    #[test]
    fn dedup_only_one_pinball_wizard_in_run_stats() {
        let mut app = test_app();
        // Pre-fill with existing PinballWizard
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::PinballWizard,
                node_index: 0,
                value: 15.0,
                detail: None,
            });
        }

        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.cell_bounces_since_breaker = 15;
        }
        app.insert_resource(TestBoltHitBreaker(vec![BoltHitBreaker {
            bolt: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let pinball_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::PinballWizard)
            .count();
        assert_eq!(
            pinball_count, 1,
            "should NOT add a second PinballWizard highlight"
        );
    }
}
