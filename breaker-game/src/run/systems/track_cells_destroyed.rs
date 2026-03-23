//! System to track total cells destroyed for run stats.

use bevy::prelude::*;

use crate::{cells::messages::CellDestroyed, run::resources::RunStats};

/// Reads [`CellDestroyed`] messages and increments [`RunStats::cells_destroyed`].
pub(crate) fn track_cells_destroyed(
    mut reader: MessageReader<CellDestroyed>,
    mut stats: ResMut<RunStats>,
) {
    for _msg in reader.read() {
        stats.cells_destroyed += 1;
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
    }
}
