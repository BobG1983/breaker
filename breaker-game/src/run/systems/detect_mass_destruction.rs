//! System to detect `MassDestruction` highlights from rapid cell destruction.

use bevy::prelude::*;

use crate::{
    cells::messages::CellDestroyed,
    run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
};

/// Reads [`CellDestroyed`] messages and detects `MassDestruction` highlights
/// when enough cells are destroyed within the configured time window.
///
/// Prunes stale timestamps, checks the count threshold, records the highlight
/// in [`RunStats`], and emits [`HighlightTriggered`].
pub(crate) fn detect_mass_destruction(
    mut reader: MessageReader<CellDestroyed>,
    time: Res<Time<Fixed>>,
    config: Res<HighlightConfig>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    let now = time.elapsed_secs();
    let window_start = now - config.mass_destruction_window_secs;

    for _msg in reader.read() {
        tracker.cell_destroyed_times.push(now);
    }

    // Prune timestamps older than the window
    tracker.cell_destroyed_times.retain(|&t| t >= window_start);

    let count = tracker.cell_destroyed_times.len();
    if count >= config.mass_destruction_count as usize {
        // Always emit HighlightTriggered for juice/VFX feedback
        writer.write(HighlightTriggered {
            kind: HighlightKind::MassDestruction,
        });

        // Only record the highlight once
        let already_recorded = stats
            .highlights
            .iter()
            .any(|h| h.kind == HighlightKind::MassDestruction);
        if !already_recorded && stats.highlights.len() < config.highlight_cap as usize {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::MassDestruction,
                node_index: run_state.node_index,
                value: f32::from(u16::try_from(count).unwrap_or(u16::MAX)),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{HighlightKind, RunHighlight};

    #[derive(Resource)]
    struct TestMessages(Vec<CellDestroyed>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<CellDestroyed>) {
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
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_messages,
                    detect_mass_destruction,
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

    fn make_cell_destroyed_batch(count: usize) -> Vec<CellDestroyed> {
        (0..count)
            .map(|_| CellDestroyed {
                was_required_to_clear: true,
            })
            .collect()
    }

    // --- Behavior 16: MassDestruction detected with enough cells in window ---

    #[test]
    fn mass_destruction_detected_with_10_cells_in_window() {
        let mut app = test_app();
        let config = HighlightConfig {
            mass_destruction_count: 10,
            mass_destruction_window_secs: 2.0,
            ..Default::default()
        };
        app.insert_resource(config);

        // Advance time to ~5.0s first so we have a concrete time reference
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let advance_ticks = u32::try_from(
            std::time::Duration::from_secs(5).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestMessages(vec![]));
        for _ in 0..advance_ticks {
            tick(&mut app);
        }

        // Send 10 CellDestroyed messages in one tick (all at time ~5.0)
        app.insert_resource(TestMessages(make_cell_destroyed_batch(10)));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let mass = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::MassDestruction);
        assert!(
            mass.is_some(),
            "should detect MassDestruction with 10 cells destroyed at time ~5.0 within window=2.0"
        );
    }

    // --- Behavior 17: Old timestamps pruned outside window ---

    #[test]
    fn old_timestamps_pruned_outside_window() {
        let mut app = test_app();
        let config = HighlightConfig {
            mass_destruction_count: 10,
            mass_destruction_window_secs: 2.0,
            ..Default::default()
        };
        app.insert_resource(config);

        // Pre-seed tracker with old timestamps at times 1.0, 1.5, 2.0, 2.5
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .cell_destroyed_times = vec![1.0, 1.5, 2.0, 2.5];

        // Advance time to ~5.0s
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let advance_ticks = u32::try_from(
            std::time::Duration::from_secs(5).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestMessages(vec![]));
        for _ in 0..advance_ticks {
            tick(&mut app);
        }

        // Send 1 cell destroyed to trigger pruning
        app.insert_resource(TestMessages(vec![CellDestroyed {
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        // All timestamps at 1.0, 1.5, 2.0, 2.5 should be pruned (< 5.0 - 2.0 = 3.0)
        // Only the new one at ~5.0 should remain
        for &t in &tracker.cell_destroyed_times {
            assert!(
                t >= 3.0,
                "timestamp {t} should have been pruned (outside window of 2.0s at time ~5.0)"
            );
        }
    }

    // --- Behavior 18: HighlightTriggered message emitted on detection ---

    #[test]
    fn highlight_triggered_message_emitted_on_mass_destruction() {
        let mut app = test_app();
        let config = HighlightConfig {
            mass_destruction_count: 10,
            mass_destruction_window_secs: 2.0,
            ..Default::default()
        };
        app.insert_resource(config);

        // Advance time to ~5.0s
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let advance_ticks = u32::try_from(
            std::time::Duration::from_secs(5).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestMessages(vec![]));
        for _ in 0..advance_ticks {
            tick(&mut app);
        }

        // Send 10 CellDestroyed messages
        app.insert_resource(TestMessages(make_cell_destroyed_batch(10)));
        tick(&mut app);

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let mass_msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::MassDestruction);
        assert!(
            mass_msg.is_some(),
            "should emit HighlightTriggered with MassDestruction kind"
        );
    }

    // --- Behavior 19: No double-record if MassDestruction already in highlights ---

    #[test]
    fn no_double_record_if_mass_destruction_already_recorded() {
        let mut app = test_app();
        let config = HighlightConfig {
            mass_destruction_count: 10,
            mass_destruction_window_secs: 2.0,
            ..Default::default()
        };
        app.insert_resource(config);

        // Pre-fill highlights with an existing MassDestruction
        app.world_mut()
            .resource_mut::<RunStats>()
            .highlights
            .push(RunHighlight {
                kind: HighlightKind::MassDestruction,
                node_index: 0,
                value: 10.0,
            });

        // Advance time to ~5.0s
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let advance_ticks = u32::try_from(
            std::time::Duration::from_secs(5).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestMessages(vec![]));
        for _ in 0..advance_ticks {
            tick(&mut app);
        }

        // Send 10 CellDestroyed messages (would normally trigger MassDestruction)
        app.insert_resource(TestMessages(make_cell_destroyed_batch(10)));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let mass_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::MassDestruction)
            .count();
        assert_eq!(
            mass_count, 1,
            "should NOT add a second MassDestruction highlight (still 1 from pre-fill)"
        );

        // But HighlightTriggered should STILL be emitted (for juice/VFX feedback)
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let mass_msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::MassDestruction);
        assert!(
            mass_msg.is_some(),
            "should still emit HighlightTriggered even when not adding to highlights"
        );
    }
}
