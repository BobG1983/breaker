use bevy::prelude::*;

pub(crate) fn fire(entity: Entity, range: f32, damage_mult: f32, world: &mut World) {
    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    // Placeholder: spatial query for all cells within range and applying
    // damage is not yet implemented.
    debug!(
        "Explode fired from {:?} at ({}, {}) — range: {}, damage_mult: {}",
        entity, position.x, position.y, range, damage_mult
    );
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

        // Placeholder fire should complete without panicking
        fire(entity, 50.0, 2.0, &mut world);
    }

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

        // reverse should complete without panicking or modifying anything
        reverse(entity, &mut world);

        // Entity still exists
        assert!(
            world.get_entity(entity).is_ok(),
            "entity should still exist after no-op reverse"
        );
    }
}
