//! System to track node completion by counting destroyed required cells.

use bevy::prelude::*;
use tracing::info;

use crate::{
    cells::messages::CellDestroyedAt,
    state::run::node::{ClearRemainingCount, messages::NodeCleared},
};

/// Reads [`CellDestroyedAt`] messages and decrements [`ClearRemainingCount`].
/// When the count reaches zero, sends [`NodeCleared`].
pub(crate) fn track_node_completion(
    mut reader: MessageReader<CellDestroyedAt>,
    mut remaining: ResMut<ClearRemainingCount>,
    mut writer: MessageWriter<NodeCleared>,
    mut fired: Local<bool>,
) {
    // Reset when a new node loads (resource re-inserted by init_clear_remaining).
    if remaining.is_changed() {
        *fired = false;
    }

    for msg in reader.read() {
        if msg.was_required_to_clear {
            remaining.remaining = remaining.remaining.saturating_sub(1);
        }
    }

    if remaining.remaining == 0 && !*fired {
        info!("node cleared");
        writer.write(NodeCleared);
        *fired = true;
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
    fn node_cleared_fires_when_remaining_starts_at_zero() {
        // Empty grids start with remaining=0. NodeCleared should fire
        // immediately on the first tick (no cells to destroy).
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
            captured.0,
            "NodeCleared should fire when remaining starts at 0 (empty grid)"
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
            was_required_to_clear: true,
        }]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 4);
    }

    // ── Counter-based capture helper for counting NodeCleared messages ────

    #[derive(Resource, Default)]
    struct NodeClearedCount(u32);

    fn capture_node_cleared_count(
        mut reader: MessageReader<NodeCleared>,
        mut count: ResMut<NodeClearedCount>,
    ) {
        for _ in reader.read() {
            count.0 += 1;
        }
    }

    #[test]
    fn node_cleared_fires_exactly_once_for_same_frame_multiple_cell_destruction() {
        let mut app = test_app(3);
        app.init_resource::<NodeClearedCount>();
        app.add_systems(
            FixedUpdate,
            capture_node_cleared_count.after(track_node_completion),
        );

        // Three required cells destroyed in the same frame
        app.insert_resource(TestMessages(vec![
            CellDestroyedAt {
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                was_required_to_clear: true,
            },
            CellDestroyedAt {
                was_required_to_clear: true,
            },
        ]));
        tick(&mut app);

        let remaining = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            remaining.remaining, 0,
            "all 3 required cells should decrement count to 0"
        );

        let cleared_count = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared_count.0, 1,
            "NodeCleared should fire exactly once, not {} times",
            cleared_count.0
        );
    }

    #[test]
    fn node_cleared_fires_once_with_mixed_required_and_non_required() {
        // 2 required + 1 non-required, remaining = 2
        let mut app = test_app(2);
        app.init_resource::<NodeClearedCount>();
        app.add_systems(
            FixedUpdate,
            capture_node_cleared_count.after(track_node_completion),
        );

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

        let remaining = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            remaining.remaining, 0,
            "only required cells should decrement: 2 - 2 = 0"
        );

        let cleared_count = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared_count.0, 1,
            "NodeCleared should fire exactly once when remaining hits 0, got {}",
            cleared_count.0
        );
    }
}
