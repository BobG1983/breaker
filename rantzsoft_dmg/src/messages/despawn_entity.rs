//! Deferred despawn request.

use bevy::prelude::*;

/// Deferred despawn request. Emitted after kill handling + death animations +
/// trigger evaluation complete. A consumer system in `FixedPostUpdate`
/// processes this queue and calls `commands.entity(e).despawn()`.
///
/// Use this instead of despawning directly in death handling — the entity
/// must survive the full death chain.
#[derive(Message, Clone, Debug)]
pub struct DespawnEntity {
    /// The entity to despawn in the next `FixedPostUpdate`.
    pub entity: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Routed through a generic `T: Clone` bound — proves a `Clone` impl exists
    // for the value type at compile time without tripping
    // `clippy::redundant_clone`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    // ── Behavior 22: `DespawnEntity` constructs with `entity` set ──

    #[test]
    fn constructs_with_entity_set() {
        let msg = DespawnEntity {
            entity: Entity::PLACEHOLDER,
        };
        assert_eq!(msg.entity, Entity::PLACEHOLDER);
    }

    // ── Behavior 23: `DespawnEntity` derives `Clone` ──

    #[test]
    fn derives_clone() {
        let original = DespawnEntity {
            entity: Entity::PLACEHOLDER,
        };
        let cloned = require_clone(&original);
        assert_eq!(cloned.entity, original.entity);
    }

    // ── Behavior 24: `DespawnEntity` derives `Debug` — formatter does not
    //     panic ──

    #[test]
    fn derives_debug_non_empty_format() {
        let msg = DespawnEntity {
            entity: Entity::PLACEHOLDER,
        };
        let s = format!("{msg:?}");
        assert!(!s.is_empty());
    }

    // ── Behavior 25: `DespawnEntity` is a Bevy `Message` — `add_message`
    //     registration succeeds ──

    #[test]
    fn add_message_registration_succeeds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DespawnEntity>();
        assert!(app.world().contains_resource::<Messages<DespawnEntity>>());
    }

    #[test]
    fn init_resource_messages_succeeds() {
        // Edge case: init_resource works directly.
        let mut world = World::new();
        world.init_resource::<Messages<DespawnEntity>>();
        assert!(world.contains_resource::<Messages<DespawnEntity>>());
    }

    // ── Behavior 26: `DespawnEntity` round-trips through `MessageWriter` →
    //     `MessageReader` ──

    #[test]
    fn round_trip_writer_to_reader() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DespawnEntity>();

        // Write directly via the resource (one of the permitted write idioms).
        app.world_mut()
            .resource_mut::<Messages<DespawnEntity>>()
            .write(DespawnEntity {
                entity: Entity::PLACEHOLDER,
            });

        app.update();

        // Read via direct resource drain (the test spec permits either a
        // registered MessageReader system or `Messages::drain()`).
        let drained: Vec<DespawnEntity> = app
            .world_mut()
            .resource_mut::<Messages<DespawnEntity>>()
            .drain()
            .collect();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].entity, Entity::PLACEHOLDER);
    }
}
