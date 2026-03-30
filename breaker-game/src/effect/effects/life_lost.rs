use bevy::prelude::*;

/// Tracks remaining lives for an entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivesCount(pub u32);

/// Decrements `LivesCount` on the entity if present and greater than zero.
pub(crate) fn fire(entity: Entity, _source_chip: &str, world: &mut World) {
    if let Some(mut lives) = world.get_mut::<LivesCount>(entity)
        && lives.0 > 0
    {
        lives.0 -= 1;
    }
}

/// Restores one life — increments `LivesCount` on the entity.
pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    if let Some(mut lives) = world.get_mut::<LivesCount>(entity) {
        lives.0 = lives.0.saturating_add(1);
    }
}

/// Registers systems for `LifeLost` effect.
pub(crate) const fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_decrements_lives_count() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(3)).id();

        fire(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 2, "LivesCount(3) should become LivesCount(2)");
    }

    #[test]
    fn fire_does_not_decrement_below_zero() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(0)).id();

        fire(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 0, "LivesCount(0) should remain 0");
    }

    #[test]
    fn reverse_restores_one_life() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(2)).id();

        reverse(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 3, "reverse should increment LivesCount by 1");
    }

    #[test]
    fn reverse_saturates_at_max_lives() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(u32::MAX)).id();

        reverse(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            u32::MAX,
            "reverse at u32::MAX should saturate, not overflow"
        );
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        reverse(entity, "", &mut world);

        assert!(
            world.get::<LivesCount>(entity).is_none(),
            "reverse on entity without LivesCount should not add the component"
        );
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, "", &mut world);

        assert!(
            world.get::<LivesCount>(entity).is_none(),
            "fire on entity without LivesCount should not add the component"
        );
    }
}
