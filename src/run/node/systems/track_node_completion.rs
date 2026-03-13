//! System to track node completion by counting destroyed required cells.

use bevy::prelude::*;

use crate::{
    cells::messages::CellDestroyed,
    run::{messages::NodeCleared, node::ClearRemainingCount},
};

/// Reads [`CellDestroyed`] messages and decrements [`ClearRemainingCount`].
/// When the count reaches zero, sends [`NodeCleared`].
pub fn track_node_completion(
    mut reader: MessageReader<CellDestroyed>,
    mut remaining: ResMut<ClearRemainingCount>,
    mut writer: MessageWriter<NodeCleared>,
) {
    for msg in reader.read() {
        if msg.was_required_to_clear {
            remaining.remaining = remaining.remaining.saturating_sub(1);
        }
    }

    if remaining.remaining == 0 && remaining.is_changed() {
        writer.write(NodeCleared);
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

    fn test_app(remaining: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<CellDestroyed>();
        app.add_message::<NodeCleared>();
        app.insert_resource(ClearRemainingCount { remaining });
        app.add_systems(
            FixedUpdate,
            (enqueue_messages, track_node_completion).chain(),
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
    fn decrement_on_required_destroyed() {
        let mut app = test_app(3);
        app.insert_resource(TestMessages(vec![CellDestroyed {
            entity: Entity::PLACEHOLDER,
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 2);
    }

    #[test]
    fn ignore_non_required_destroyed() {
        let mut app = test_app(3);
        app.insert_resource(TestMessages(vec![CellDestroyed {
            entity: Entity::PLACEHOLDER,
            was_required_to_clear: false,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 3);
    }

    #[test]
    fn node_cleared_fires_when_remaining_hits_zero() {
        let mut app = test_app(1);
        app.insert_resource(TestMessages(vec![CellDestroyed {
            entity: Entity::PLACEHOLDER,
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 0);
    }

    #[test]
    fn node_cleared_does_not_fire_while_cells_remain() {
        let mut app = test_app(5);
        app.insert_resource(TestMessages(vec![CellDestroyed {
            entity: Entity::PLACEHOLDER,
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 4);
    }
}
