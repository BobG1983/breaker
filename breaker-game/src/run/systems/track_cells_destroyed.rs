//! System to track total cells destroyed for run stats.

use bevy::prelude::*;

use crate::{cells::messages::CellDestroyedAt, run::resources::RunStats};

/// Reads [`CellDestroyedAt`] messages and increments
/// [`RunStats::cells_destroyed`].
pub(crate) fn track_cells_destroyed(
    mut reader: MessageReader<CellDestroyedAt>,
    mut stats: ResMut<RunStats>,
) {
    for _msg in reader.read() {
        stats.cells_destroyed += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::messages::CellDestroyedAt;

    #[derive(Resource)]
    struct TestMessages(Vec<CellDestroyedAt>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<CellDestroyedAt>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyedAt>()
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

    // =========================================================================
    // C7 Wave 2a: CellDestroyed -> CellDestroyedAt migration (behavior 32b)
    // =========================================================================

    #[derive(Resource)]
    struct TestCellDestroyedAtMsgs(Vec<crate::cells::messages::CellDestroyedAt>);

    fn enqueue_cell_destroyed_at(
        msg_res: Res<TestCellDestroyedAtMsgs>,
        mut writer: MessageWriter<crate::cells::messages::CellDestroyedAt>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app_cell_destroyed_at() -> App {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyedAt>()
            .init_resource::<RunStats>()
            .add_systems(
                FixedUpdate,
                (enqueue_cell_destroyed_at, track_cells_destroyed).chain(),
            );
        app
    }

    #[test]
    fn track_cells_destroyed_reads_cell_destroyed_at() {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = test_app_cell_destroyed_at();
        app.insert_resource(TestCellDestroyedAtMsgs(vec![
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: false,
            },
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: true,
            },
        ]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.cells_destroyed, 3,
            "all 3 CellDestroyedAt messages should increment cells_destroyed"
        );
    }

    #[test]
    fn increments_cells_destroyed_for_each_message() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: false,
            },
            CellDestroyedAt {
                position: Vec2::ZERO,
                was_required_to_clear: true,
            },
        ]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.cells_destroyed, 3,
            "all 3 CellDestroyedAt messages should increment cells_destroyed"
        );
    }
}
