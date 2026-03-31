//! Despawns bolt entities when `RequestBoltDestroyed` messages are received.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::bolt::messages::RequestBoltDestroyed;

/// Despawns bolt entities from [`RequestBoltDestroyed`] messages.
///
/// Deduplicates entity IDs to avoid double-despawn when multiple systems
/// request destruction of the same bolt in the same frame (e.g., lifespan
/// expiry + out-of-bounds in the same tick).
pub(crate) fn cleanup_destroyed_bolts(
    mut reader: MessageReader<RequestBoltDestroyed>,
    mut commands: Commands,
) {
    let mut to_despawn: HashSet<Entity> = HashSet::new();
    for msg in reader.read() {
        to_despawn.insert(msg.bolt);
    }
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::messages::RequestBoltDestroyed;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    #[derive(Resource)]
    struct SendRequestBoltDestroyed(Option<RequestBoltDestroyed>);

    fn enqueue_request(
        msg: Res<SendRequestBoltDestroyed>,
        mut writer: MessageWriter<RequestBoltDestroyed>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .insert_resource(SendRequestBoltDestroyed(None))
            .add_systems(
                FixedUpdate,
                (enqueue_request, cleanup_destroyed_bolts).chain(),
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
    fn cleanup_destroyed_bolts_despawns_bolt() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<SendRequestBoltDestroyed>().0 =
            Some(RequestBoltDestroyed { bolt });

        tick(&mut app);

        assert!(
            app.world().get_entity(bolt).is_err(),
            "bolt entity should be despawned after cleanup_destroyed_bolts processes \
             RequestBoltDestroyed"
        );
    }

    #[test]
    fn cleanup_destroyed_bolts_noop_without_message() {
        let mut app = test_app();

        let bolt = app.world_mut().spawn_empty().id();
        // No message written -- resource stays None.

        tick(&mut app);

        assert!(
            app.world().get_entity(bolt).is_ok(),
            "bolt entity should still exist when no RequestBoltDestroyed message is sent"
        );
    }
}
