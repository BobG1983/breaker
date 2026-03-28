use bevy::prelude::*;

/// Tracks accumulated damage bonus from consecutive cell hits.
#[derive(Component, Debug, Clone)]
pub struct RampingDamageState {
    /// Damage bonus added per consecutive cell hit.
    pub bonus_per_hit: f32,
    /// Total accumulated damage bonus.
    pub accumulated: f32,
    /// Number of consecutive hits tracked.
    pub hits: u32,
}

pub(crate) fn fire(entity: Entity, bonus_per_hit: f32, world: &mut World) {
    world.entity_mut(entity).insert(RampingDamageState {
        bonus_per_hit,
        accumulated: 0.0,
        hits: 0,
    });
}

pub(crate) fn reverse(entity: Entity, world: &mut World) {
    world.entity_mut(entity).remove::<RampingDamageState>();
}

pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_inserts_ramping_damage_state_with_zero_accumulated() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 1.5, &mut world);

        let state = world.get::<RampingDamageState>(entity).unwrap();
        assert!(
            (state.bonus_per_hit - 1.5).abs() < f32::EPSILON,
            "expected bonus_per_hit 1.5, got {}",
            state.bonus_per_hit
        );
        assert!(
            (state.accumulated - 0.0).abs() < f32::EPSILON,
            "expected accumulated 0.0, got {}",
            state.accumulated
        );
        assert_eq!(state.hits, 0);
    }

    #[test]
    fn fire_overwrites_existing_state_fresh_start() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 1.5, &mut world);

        // Simulate some accumulated state
        {
            let mut state = world.get_mut::<RampingDamageState>(entity).unwrap();
            state.accumulated = 10.0;
            state.hits = 5;
        }

        // Fire again — should overwrite with fresh state
        fire(entity, 2.0, &mut world);

        let state = world.get::<RampingDamageState>(entity).unwrap();
        assert!(
            (state.bonus_per_hit - 2.0).abs() < f32::EPSILON,
            "expected bonus_per_hit 2.0, got {}",
            state.bonus_per_hit
        );
        assert!(
            (state.accumulated - 0.0).abs() < f32::EPSILON,
            "accumulated should be reset, got {}",
            state.accumulated
        );
        assert_eq!(state.hits, 0, "hits should be reset");
    }

    #[test]
    fn reverse_removes_ramping_damage_state() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 1.5, &mut world);
        assert!(world.get::<RampingDamageState>(entity).is_some());

        reverse(entity, &mut world);

        assert!(
            world.get::<RampingDamageState>(entity).is_none(),
            "ramping damage state should be removed after reverse"
        );
    }
}
