use bevy::prelude::*;

/// Marks the breaker as having `FlashStep` active — the dash system should
/// teleport on reverse-direction dash during settling instead of sliding.
#[derive(Component)]
pub struct FlashStepActive;

/// Inserts `FlashStepActive` marker on the entity.
pub(crate) fn fire(entity: Entity, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if world.get::<FlashStepActive>(entity).is_some() {
        return;
    }

    world.entity_mut(entity).insert(FlashStepActive);
}

/// Removes `FlashStepActive` from the entity (with despawn guard).
pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    world.entity_mut(entity).remove::<FlashStepActive>();
}

/// No runtime systems needed for `FlashStep`.
pub(crate) const fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: fire() inserts FlashStepActive marker on entity ──

    #[test]
    fn fire_inserts_flash_step_active_on_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "fire() should insert FlashStepActive on the entity"
        );
    }

    #[test]
    fn fire_does_not_disturb_existing_components() {
        // Edge case: entity has other components — FlashStepActive is still
        // inserted without disturbing them.
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

        fire(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "fire() should insert FlashStepActive even when entity has other components"
        );
        let transform = world.get::<Transform>(entity).unwrap();
        assert!(
            (transform.translation.x - 10.0).abs() < f32::EPSILON,
            "fire() should not disturb existing Transform"
        );
        assert!(
            (transform.translation.y - 20.0).abs() < f32::EPSILON,
            "fire() should not disturb existing Transform"
        );
    }

    // ── Behavior 2: fire() on entity that already has FlashStepActive is idempotent ──

    #[test]
    fn fire_is_idempotent_on_entity_with_flash_step_active() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, "", &mut world);
        fire(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "fire() called twice should still leave exactly one FlashStepActive"
        );
        // Entity must remain valid (no panic from double insert)
        assert!(
            world.get_entity(entity).is_ok(),
            "entity should remain valid after double fire()"
        );
    }

    #[test]
    fn fire_three_times_in_succession_remains_valid() {
        // Edge case: call fire() three times — entity still has exactly one
        // FlashStepActive and remains valid.
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, "", &mut world);
        fire(entity, "", &mut world);
        fire(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_some(),
            "fire() called three times should still have FlashStepActive"
        );
        assert!(
            world.get_entity(entity).is_ok(),
            "entity should remain valid after triple fire()"
        );
    }

    // ── Behavior 3: reverse() removes FlashStepActive from entity ──

    #[test]
    fn reverse_removes_flash_step_active() {
        let mut world = World::new();
        let entity = world.spawn(FlashStepActive).id();

        reverse(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "reverse() should remove FlashStepActive from the entity"
        );
    }

    #[test]
    fn reverse_preserves_other_components() {
        // Edge case: entity has other components alongside FlashStepActive —
        // only FlashStepActive is removed, other components remain.
        let mut world = World::new();
        let entity = world
            .spawn((FlashStepActive, Transform::from_xyz(5.0, 15.0, 0.0)))
            .id();

        reverse(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "reverse() should remove FlashStepActive"
        );
        let transform = world.get::<Transform>(entity).unwrap();
        assert!(
            (transform.translation.x - 5.0).abs() < f32::EPSILON,
            "reverse() should not disturb existing Transform"
        );
        assert!(
            (transform.translation.y - 15.0).abs() < f32::EPSILON,
            "reverse() should not disturb existing Transform"
        );
    }

    // ── Behavior 4: reverse() on entity without FlashStepActive is a no-op ──

    #[test]
    fn reverse_without_flash_step_active_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Should not panic
        reverse(entity, "", &mut world);

        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "reverse() on entity without FlashStepActive should remain None"
        );
    }

    #[test]
    fn reverse_on_freshly_spawned_empty_entity_does_not_panic() {
        // Edge case: entity is freshly spawned with spawn_empty()
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        reverse(entity, "", &mut world); // must not panic
    }

    // ── Behavior 5: reverse() on despawned entity does not panic ──

    #[test]
    fn reverse_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn(FlashStepActive).id();
        world.despawn(entity);

        // Stale entity ID — must not panic
        reverse(entity, "", &mut world);
    }

    #[test]
    fn reverse_on_never_spawned_entity_does_not_panic() {
        // Edge case: entity was never spawned
        let mut world = World::new();
        let stale = Entity::from_raw_u32(9999).unwrap();

        reverse(stale, "", &mut world);
    }

    // ── Behavior 6: fire() on despawned entity does not panic ──

    #[test]
    fn fire_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        // Stale entity ID — must not panic, must not insert anywhere
        fire(entity, "", &mut world);
    }

    #[test]
    fn fire_on_never_spawned_entity_does_not_panic() {
        // Edge case: entity was never spawned
        let mut world = World::new();
        let stale = Entity::from_raw_u32(9999).unwrap();

        fire(stale, "", &mut world);
    }

    // ── Behavior 7: register() is a no-op const fn ──

    #[test]
    fn register_is_noop_app_runs_without_error() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        register(&mut app);

        app.update(); // must not panic
    }

    #[test]
    fn register_called_multiple_times_does_not_panic() {
        // Edge case: calling register() multiple times
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        register(&mut app);
        register(&mut app);
        register(&mut app);

        app.update(); // must not panic
    }
}
