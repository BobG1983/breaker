//! Despawn-request processor.
//!
//! Reads `DespawnEntity` messages and issues `commands.entity(e).try_despawn()`
//! for each. This is the ONLY system in the crate that despawns entities.
//! Runs in `FixedPostUpdate`, wired by `RantzDmgPlugin::build`.

use bevy::prelude::*;

use crate::messages::DespawnEntity;

/// Drain `DespawnEntity` messages and despawn each entity. Uses
/// `try_despawn()` so a double-queued despawn (the same entity submitted
/// twice in one tick) is tolerated — the second call silently no-ops.
pub(crate) fn process_despawn_requests(
    mut reader: MessageReader<DespawnEntity>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.entity).try_despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::{RantzDmgPlugin, messages::DespawnEntity};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn enqueue(app: &mut App, entity: Entity) {
        app.world_mut()
            .resource_mut::<Messages<DespawnEntity>>()
            .write(DespawnEntity { entity });
    }

    // ── Behavior 135: DespawnEntity message despawns the referenced entity ──

    #[test]
    fn single_message_despawns_entity() {
        let mut app = test_app();
        let e = app.world_mut().spawn_empty().id();

        enqueue(&mut app, e);
        tick(&mut app);

        assert!(
            app.world().get_entity(e).is_err(),
            "entity should be despawned after tick"
        );
    }

    #[test]
    fn two_messages_despawn_two_entities() {
        // Edge case 135a.
        let mut app = test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();

        enqueue(&mut app, e1);
        enqueue(&mut app, e2);
        tick(&mut app);

        assert!(app.world().get_entity(e1).is_err());
        assert!(app.world().get_entity(e2).is_err());
    }

    // ── Behavior 136: no pending messages → entity still present ──

    #[test]
    fn no_messages_entity_still_present() {
        let mut app = test_app();
        let e = app.world_mut().spawn_empty().id();

        tick(&mut app);

        assert!(app.world().get_entity(e).is_ok());
    }

    // ── Behavior 137: tolerates already-despawned entity (try_despawn) ──

    #[test]
    fn tolerates_already_despawned_entity() {
        let mut app = test_app();
        let e = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(e);

        enqueue(&mut app, e);
        tick(&mut app); // must not panic.

        assert!(app.world().get_entity(e).is_err());
    }

    // ── Behavior 138: duplicate DespawnEntity for same entity does not panic ──

    #[test]
    fn duplicate_despawn_for_same_entity_is_safe() {
        let mut app = test_app();
        let e = app.world_mut().spawn_empty().id();

        enqueue(&mut app, e);
        enqueue(&mut app, e);
        tick(&mut app);

        assert!(app.world().get_entity(e).is_err());
    }
}
