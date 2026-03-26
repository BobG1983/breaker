//! Despawns cell entities when `RequestCellDestroyed` messages are received.

use bevy::prelude::*;

use crate::cells::messages::RequestCellDestroyed;

/// Despawns cell entities from [`RequestCellDestroyed`] messages.
///
/// Runs after all bridges have finished evaluating the entity's data.
pub(crate) fn cleanup_destroyed_cells(
    mut reader: MessageReader<RequestCellDestroyed>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.cell).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::messages::RequestCellDestroyed;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    #[derive(Resource)]
    struct SendRequestCellDestroyed(Option<RequestCellDestroyed>);

    fn enqueue_request(
        msg: Res<SendRequestCellDestroyed>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestCellDestroyed>()
            .insert_resource(SendRequestCellDestroyed(None))
            .add_systems(
                FixedUpdate,
                (enqueue_request, cleanup_destroyed_cells).chain(),
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

    // ---------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------

    #[test]
    fn cleanup_destroyed_cells_despawns_cell() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendRequestCellDestroyed>().0 =
            Some(RequestCellDestroyed { cell });

        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_err(),
            "cell entity should be despawned after cleanup_destroyed_cells processes \
             RequestCellDestroyed"
        );
    }

    #[test]
    fn cleanup_destroyed_cells_noop_without_message() {
        let mut app = test_app();

        let cell = app.world_mut().spawn_empty().id();
        // No message written -- resource stays None.

        tick(&mut app);

        assert!(
            app.world().get_entity(cell).is_ok(),
            "cell entity should still exist when no RequestCellDestroyed message is sent"
        );
    }
}
