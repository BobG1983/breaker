//! System to track total cells destroyed for run stats.

use bevy::prelude::*;

use crate::prelude::*;

/// Reads [`Destroyed<Cell>`] messages and increments
/// [`RunStats::cells_destroyed`].
pub(crate) fn track_cells_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    mut stats: ResMut<RunStats>,
) {
    for _msg in reader.read() {
        stats.cells_destroyed += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;

    #[derive(Resource)]
    struct TestMessages(Vec<Destroyed<Cell>>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<Destroyed<Cell>>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Cell>>()
            .with_resource::<RunStats>()
            .with_system(
                FixedUpdate,
                (enqueue_messages, track_cells_destroyed).chain(),
            )
            .build()
    }

    fn make_destroyed() -> Destroyed<Cell> {
        Destroyed::<Cell> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        }
    }

    #[test]
    fn increments_cells_destroyed_for_each_message() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            make_destroyed(),
            make_destroyed(),
            make_destroyed(),
        ]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.cells_destroyed, 3,
            "all 3 Destroyed<Cell> messages should increment cells_destroyed"
        );
    }
}
