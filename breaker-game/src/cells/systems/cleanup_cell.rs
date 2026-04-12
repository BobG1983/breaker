//! Emits [`CellDestroyedAt`] and despawns cell entities from
//! [`RequestCellDestroyed`] messages.

use bevy::prelude::*;

use crate::cells::messages::{CellDestroyedAt, RequestCellDestroyed};

/// Emits [`CellDestroyedAt`] and despawns cell entities from [`RequestCellDestroyed`] messages.
///
/// Runs after all bridges have finished evaluating the entity's data.
pub(crate) fn cleanup_cell(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut commands: Commands,
    mut writer: MessageWriter<CellDestroyedAt>,
) {
    for msg in reader.read() {
        writer.write(CellDestroyedAt {
            was_required_to_clear: msg.was_required_to_clear,
        });
        commands.entity(msg.cell).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::test_utils::tick;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    /// Resource holding messages to enqueue before each tick.
    #[derive(Resource, Default)]
    struct EnqueueRequestCellDestroyed(Vec<RequestCellDestroyed>);

    /// System that writes all queued `RequestCellDestroyed` messages.
    fn enqueue_requests(
        mut msg_res: ResMut<EnqueueRequestCellDestroyed>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        for msg in msg_res.0.drain(..) {
            writer.write(msg);
        }
    }

    /// Resource that captures emitted `CellDestroyedAt` messages for assertion.
    #[derive(Resource, Default)]
    struct CapturedCellDestroyedAt(Vec<CellDestroyedAt>);

    /// System that reads all `CellDestroyedAt` messages and stores them.
    fn capture_cell_destroyed_at(
        mut reader: MessageReader<CellDestroyedAt>,
        mut captured: ResMut<CapturedCellDestroyedAt>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;

        TestAppBuilder::new()
            .with_message::<RequestCellDestroyed>()
            .with_message::<CellDestroyedAt>()
            .with_resource::<EnqueueRequestCellDestroyed>()
            .with_resource::<CapturedCellDestroyedAt>()
            .with_system(
                FixedUpdate,
                (enqueue_requests, cleanup_cell, capture_cell_destroyed_at).chain(),
            )
            .build()
    }

    // ---------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------

    /// Behavior 1: `cleanup_cell` fires `CellDestroyedAt` with correct fields.
    ///
    /// Given: Cell entity exists, `RequestCellDestroyed` with `was_required_to_clear` = true
    /// When: `cleanup_cell` runs
    /// Then: `CellDestroyedAt` emitted with `was_required_to_clear` = true
    #[test]
    fn cleanup_cell_fires_cell_destroyed_at_with_correct_fields() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<EnqueueRequestCellDestroyed>()
            .0
            .push(RequestCellDestroyed {
                cell,
                was_required_to_clear: true,
            });

        tick(&mut app);

        let captured = app.world().resource::<CapturedCellDestroyedAt>();
        assert_eq!(
            captured.0.len(),
            1,
            "expected exactly one CellDestroyedAt message, got {}",
            captured.0.len()
        );
        assert!(
            captured.0[0].was_required_to_clear,
            "CellDestroyedAt was_required_to_clear should be true"
        );
    }

    /// Behavior 2: `cleanup_cell` fires `CellDestroyedAt` for non-required cells.
    ///
    /// Given: Cell entity exists, `RequestCellDestroyed` with `was_required_to_clear` = false
    /// When: `cleanup_cell` runs
    /// Then: `CellDestroyedAt` emitted with `was_required_to_clear` = false
    #[test]
    fn cleanup_cell_fires_cell_destroyed_at_for_non_required_cell() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<EnqueueRequestCellDestroyed>()
            .0
            .push(RequestCellDestroyed {
                cell,
                was_required_to_clear: false,
            });

        tick(&mut app);

        let captured = app.world().resource::<CapturedCellDestroyedAt>();
        assert_eq!(
            captured.0.len(),
            1,
            "expected exactly one CellDestroyedAt message, got {}",
            captured.0.len()
        );
        assert!(
            !captured.0[0].was_required_to_clear,
            "CellDestroyedAt was_required_to_clear should be false"
        );
    }

    /// Behavior 3: `cleanup_cell` still despawns the cell entity.
    ///
    /// Given: Cell entity exists, `RequestCellDestroyed` sent
    /// When: `cleanup_cell` runs
    /// Then: Cell entity is despawned
    #[test]
    fn cleanup_cell_despawns_cell_entity() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<EnqueueRequestCellDestroyed>()
            .0
            .push(RequestCellDestroyed {
                cell,
                was_required_to_clear: false,
            });

        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned after cleanup_cell processes RequestCellDestroyed"
        );
    }

    /// Behavior 4: `cleanup_cell` fires `CellDestroyedAt` for each `RequestCellDestroyed`.
    ///
    /// Given: Two cell entities exist, two `RequestCellDestroyed` messages sent
    ///        (different `was_required_to_clear` values)
    /// When: `cleanup_cell` runs
    /// Then: Two `CellDestroyedAt` messages emitted with matching fields
    #[test]
    fn cleanup_cell_fires_cell_destroyed_at_for_each_request() {
        let mut app = test_app();

        let cell_a = app.world_mut().spawn_empty().id();
        let cell_b = app.world_mut().spawn_empty().id();

        {
            let mut enqueue = app
                .world_mut()
                .resource_mut::<EnqueueRequestCellDestroyed>();
            enqueue.0.push(RequestCellDestroyed {
                cell:                  cell_a,
                was_required_to_clear: true,
            });
            enqueue.0.push(RequestCellDestroyed {
                cell:                  cell_b,
                was_required_to_clear: false,
            });
        }

        tick(&mut app);

        let captured = app.world().resource::<CapturedCellDestroyedAt>();
        assert_eq!(
            captured.0.len(),
            2,
            "expected two CellDestroyedAt messages, got {}",
            captured.0.len()
        );

        // Messages should correspond to the two requests (order matches iteration order)
        let has_required = captured.0.iter().any(|m| m.was_required_to_clear);
        let has_non_required = captured.0.iter().any(|m| !m.was_required_to_clear);

        assert!(
            has_required,
            "should contain CellDestroyedAt with was_required_to_clear = true"
        );
        assert!(
            has_non_required,
            "should contain CellDestroyedAt with was_required_to_clear = false"
        );
    }

    /// Behavior 5: `cleanup_cell` is no-op when no messages are pending.
    ///
    /// Given: Cell entity exists, no `RequestCellDestroyed` message sent
    /// When: `cleanup_cell` runs
    /// Then: No `CellDestroyedAt` emitted, cell entity still exists
    #[test]
    fn cleanup_cell_noop_without_message() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        // No message enqueued -- resource stays empty.

        tick(&mut app);

        let captured = app.world().resource::<CapturedCellDestroyedAt>();
        assert!(
            captured.0.is_empty(),
            "no CellDestroyedAt should be emitted when no RequestCellDestroyed is sent"
        );
        assert!(
            app.world().get_entity(cell).is_ok(),
            "cell entity should still exist when no RequestCellDestroyed message is sent"
        );
    }
}
