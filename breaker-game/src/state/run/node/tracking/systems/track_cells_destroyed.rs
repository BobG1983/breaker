//! System to track total cells destroyed for run stats.

use bevy::prelude::*;

use crate::{cells::messages::CellDestroyedAt, state::run::resources::RunStats};

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
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .with_message::<CellDestroyedAt>()
            .with_resource::<RunStats>()
            .with_system(
                FixedUpdate,
                (enqueue_messages, track_cells_destroyed).chain(),
            )
            .build()
    }

    use crate::shared::test_utils::tick;

    #[test]
    fn increments_cells_destroyed_for_each_message() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            CellDestroyedAt {
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                was_required_to_clear: false,
            },
            CellDestroyedAt {
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
