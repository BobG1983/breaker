use bevy::prelude::*;

/// Tracks accumulated damage bonus from trigger activations.
#[derive(Component, Debug, Clone)]
pub struct RampingDamageState {
    /// Damage bonus added per trigger activation.
    pub damage_per_trigger: f32,
    /// Total accumulated damage bonus.
    pub accumulated: f32,
    /// Number of times this effect has been triggered.
    pub trigger_count: u32,
}

pub(crate) fn fire(entity: Entity, damage_per_trigger: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut state) = world.get_mut::<RampingDamageState>(entity) {
        state.accumulated += damage_per_trigger;
        state.trigger_count += 1;
    } else {
        world.entity_mut(entity).insert(RampingDamageState {
            damage_per_trigger,
            accumulated: damage_per_trigger,
            trigger_count: 1,
        });
    }
}

pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    world.entity_mut(entity).remove::<RampingDamageState>();
}

pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_inserts_fresh_state_when_no_ramping_damage_state_exists() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 1.5, "", &mut world);

        let state = world
            .get::<RampingDamageState>(entity)
            .expect("fire should insert RampingDamageState when absent");
        assert!(
            (state.damage_per_trigger - 1.5).abs() < f32::EPSILON,
            "expected damage_per_trigger 1.5, got {}",
            state.damage_per_trigger
        );
        assert!(
            (state.accumulated - 1.5).abs() < f32::EPSILON,
            "expected accumulated 1.5, got {}",
            state.accumulated
        );
        assert_eq!(
            state.trigger_count, 1,
            "expected trigger_count 1, got {}",
            state.trigger_count
        );
    }

    #[test]
    fn fire_increments_existing_state() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Pre-insert state simulating 2 prior triggers
        world.entity_mut(entity).insert(RampingDamageState {
            damage_per_trigger: 1.5,
            accumulated: 3.0,
            trigger_count: 2,
        });

        fire(entity, 1.5, "", &mut world);

        let state = world
            .get::<RampingDamageState>(entity)
            .expect("fire should preserve existing RampingDamageState");
        assert!(
            (state.accumulated - 4.5).abs() < f32::EPSILON,
            "expected accumulated 4.5 (3.0 + 1.5), got {}",
            state.accumulated
        );
        assert_eq!(
            state.trigger_count, 3,
            "expected trigger_count 3 (2 + 1), got {}",
            state.trigger_count
        );
        assert!(
            (state.damage_per_trigger - 1.5).abs() < f32::EPSILON,
            "damage_per_trigger should be unchanged at 1.5, got {}",
            state.damage_per_trigger
        );
    }

    #[test]
    fn multi_call_accumulation_is_linear() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // First call inserts fresh state
        fire(entity, 0.5, "", &mut world);

        // Three more calls should increment linearly
        fire(entity, 0.5, "", &mut world);
        fire(entity, 0.5, "", &mut world);
        fire(entity, 0.5, "", &mut world);

        let state = world
            .get::<RampingDamageState>(entity)
            .expect("entity should have RampingDamageState after multiple fires");
        assert!(
            (state.accumulated - 2.0).abs() < f32::EPSILON,
            "expected accumulated 2.0 (0.5 * 4 triggers), got {}",
            state.accumulated
        );
        assert_eq!(
            state.trigger_count, 4,
            "expected trigger_count 4 (4 triggers total), got {}",
            state.trigger_count
        );
    }

    #[test]
    fn reverse_removes_entire_ramping_damage_state() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        world.entity_mut(entity).insert(RampingDamageState {
            damage_per_trigger: 1.5,
            accumulated: 10.0,
            trigger_count: 5,
        });

        reverse(entity, "", &mut world);

        assert!(
            world.get::<RampingDamageState>(entity).is_none(),
            "ramping damage state should be removed entirely after reverse"
        );
    }

    #[test]
    fn reverse_is_noop_on_entity_without_state() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Should not panic
        reverse(entity, "", &mut world);

        assert!(
            world.get::<RampingDamageState>(entity).is_none(),
            "entity should remain without RampingDamageState"
        );
    }
}
