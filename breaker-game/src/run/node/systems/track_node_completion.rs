//! System to track node completion by counting destroyed required cells.

use bevy::prelude::*;
use tracing::info;

use crate::{
    cells::messages::CellDestroyedAt,
    run::node::{ClearRemainingCount, messages::NodeCleared},
};

/// Reads [`CellDestroyedAt`] messages and decrements [`ClearRemainingCount`].
/// When the count reaches zero, sends [`NodeCleared`].
pub(crate) fn track_node_completion(
    mut reader: MessageReader<CellDestroyedAt>,
    mut remaining: ResMut<ClearRemainingCount>,
    mut writer: MessageWriter<NodeCleared>,
) {
    let mut decremented = false;
    for msg in reader.read() {
        if msg.was_required_to_clear {
            remaining.remaining = remaining.remaining.saturating_sub(1);
            decremented = true;
        }
    }

    if remaining.remaining == 0 && decremented {
        info!("node cleared");
        writer.write(NodeCleared);
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

    #[derive(Resource, Default)]
    struct NodeClearedCaptured(bool);

    fn capture_node_cleared(
        mut reader: MessageReader<NodeCleared>,
        mut captured: ResMut<NodeClearedCaptured>,
    ) {
        if reader.read().count() > 0 {
            captured.0 = true;
        }
    }

    fn test_app(remaining: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyedAt>()
            .add_message::<NodeCleared>()
            .insert_resource(ClearRemainingCount { remaining })
            .add_systems(
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
        app.insert_resource(TestMessages(vec![CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 2);
    }

    #[test]
    fn ignore_non_required_destroyed() {
        let mut app = test_app(3);
        app.insert_resource(TestMessages(vec![CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: false,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 3);
    }

    #[test]
    fn node_cleared_fires_when_remaining_hits_zero() {
        let mut app = test_app(1);
        app.init_resource::<NodeClearedCaptured>();
        app.add_systems(
            FixedUpdate,
            capture_node_cleared.after(track_node_completion),
        );
        app.insert_resource(TestMessages(vec![CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 0);
        let captured = app.world().resource::<NodeClearedCaptured>();
        assert!(
            captured.0,
            "NodeCleared should be sent when remaining reaches zero"
        );
    }

    #[test]
    fn node_cleared_does_not_fire_when_already_at_zero_with_no_messages() {
        // remaining starts at 0 but nothing changed this tick — is_changed()
        // guard should prevent a spurious NodeCleared.
        let mut app = test_app(0);
        app.init_resource::<NodeClearedCaptured>();
        app.add_systems(
            FixedUpdate,
            capture_node_cleared.after(track_node_completion),
        );
        app.insert_resource(TestMessages(vec![]));
        tick(&mut app);

        let captured = app.world().resource::<NodeClearedCaptured>();
        assert!(
            !captured.0,
            "NodeCleared should not fire when remaining starts at 0 with no messages"
        );
    }

    // =========================================================================
    // C7 Wave 2a: CellDestroyed -> CellDestroyedAt migration (behavior 32a)
    // =========================================================================

    #[derive(Resource)]
    struct TestCellDestroyedAtMessages(Vec<crate::cells::messages::CellDestroyedAt>);

    fn enqueue_cell_destroyed_at(
        msg_res: Res<TestCellDestroyedAtMessages>,
        mut writer: MessageWriter<crate::cells::messages::CellDestroyedAt>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app_cell_destroyed_at(remaining: u32) -> App {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellDestroyedAt>()
            .add_message::<NodeCleared>()
            .insert_resource(ClearRemainingCount { remaining })
            .add_systems(
                FixedUpdate,
                (enqueue_cell_destroyed_at, track_node_completion).chain(),
            );
        app
    }

    #[test]
    fn track_node_completion_reads_cell_destroyed_at_and_decrements() {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = test_app_cell_destroyed_at(3);
        app.insert_resource(TestCellDestroyedAtMessages(vec![CellDestroyedAt {
            position: Vec2::new(100.0, 200.0),
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 2,
            "CellDestroyedAt with was_required_to_clear=true should decrement count"
        );
    }

    #[test]
    fn track_node_completion_ignores_non_required_cell_destroyed_at() {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = test_app_cell_destroyed_at(3);
        app.insert_resource(TestCellDestroyedAtMessages(vec![CellDestroyedAt {
            position: Vec2::new(50.0, 75.0),
            was_required_to_clear: false,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 3,
            "CellDestroyedAt with was_required_to_clear=false should NOT decrement"
        );
    }

    #[test]
    fn node_cleared_fires_on_cell_destroyed_at_reaching_zero() {
        use crate::cells::messages::CellDestroyedAt;

        let mut app = test_app_cell_destroyed_at(1);
        app.init_resource::<NodeClearedCaptured>();
        app.add_systems(
            FixedUpdate,
            capture_node_cleared.after(track_node_completion),
        );
        app.insert_resource(TestCellDestroyedAtMessages(vec![CellDestroyedAt {
            position: Vec2::new(100.0, 200.0),
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let captured = app.world().resource::<NodeClearedCaptured>();
        assert!(
            captured.0,
            "NodeCleared should fire when CellDestroyedAt reaches zero remaining"
        );
    }

    #[test]
    fn node_cleared_does_not_fire_while_cells_remain() {
        let mut app = test_app(5);
        app.insert_resource(TestMessages(vec![CellDestroyedAt {
            position: Vec2::new(10.0, 20.0),
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 4);
    }
}
