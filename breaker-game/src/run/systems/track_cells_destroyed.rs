//! System to track cell destruction stats for the current run.

use bevy::prelude::*;

use crate::{
    cells::messages::CellDestroyed,
    run::resources::{HighlightTracker, RunStats},
};

/// Reads [`CellDestroyed`] messages and updates [`RunStats`] and
/// [`HighlightTracker`] accordingly.
pub(crate) fn track_cells_destroyed(
    mut reader: MessageReader<CellDestroyed>,
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
    time: Res<Time<Fixed>>,
) {
    for _msg in reader.read() {
        stats.cells_destroyed += 1;
        tracker.cell_destroyed_times.push(time.elapsed_secs());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct TestMessages(Vec<CellDestroyed>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<CellDestroyed>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyed>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .add_systems(
                FixedUpdate,
                (enqueue_messages, track_cells_destroyed).chain(),
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

    #[test]
    fn increments_cells_destroyed_for_each_message() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            CellDestroyed {
                was_required_to_clear: true,
            },
            CellDestroyed {
                was_required_to_clear: false,
            },
            CellDestroyed {
                was_required_to_clear: true,
            },
        ]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.cells_destroyed, 3,
            "all 3 CellDestroyed messages should increment cells_destroyed"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.cell_destroyed_times.len(),
            3,
            "all 3 destruction timestamps should be recorded"
        );
    }

    #[test]
    fn pushes_simulation_time_to_cell_destroyed_times() {
        let mut app = test_app();
        // Advance fixed time to approximately 2.5s by accumulating many timesteps
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        // 2.5s / timestep — safe conversion for test values
        let ticks_needed = u32::try_from(
            std::time::Duration::from_millis(2500).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestMessages(vec![]));
        for _ in 0..ticks_needed {
            tick(&mut app);
        }

        // Now send one destruction message
        app.insert_resource(TestMessages(vec![CellDestroyed {
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(tracker.cell_destroyed_times.len(), 1);
        let recorded_time = tracker.cell_destroyed_times[0];
        assert!(
            (recorded_time - 2.5).abs() < 0.1,
            "expected destruction time near 2.5s, got {recorded_time}"
        );
    }
}
