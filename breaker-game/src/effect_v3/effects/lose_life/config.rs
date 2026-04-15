//! `LoseLifeConfig` — fire-and-forget life decrement.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, prelude::*};

/// Decrements the entity's Hp by 1. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoseLifeConfig {}

impl Fireable for LoseLifeConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if let Some(mut hp) = world.get_mut::<Hp>(entity) {
            hp.current = (hp.current - 1.0).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect_v3::traits::Fireable;

    #[test]
    fn fire_decrements_hp_by_one() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(5.0)).id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().current - 4.0).abs() < f32::EPSILON,
            "Hp should be decremented from 5.0 to 4.0"
        );
    }

    #[test]
    fn fire_works_for_arbitrary_starting_values() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(3.0)).id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().current - 2.0).abs() < f32::EPSILON,
            "Hp should be decremented from 3.0 to 2.0"
        );
    }

    #[test]
    fn fire_leaves_zero_hp_when_hp_is_one() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(1.0)).id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().current).abs() < f32::EPSILON,
            "Hp should be 0.0 after decrementing from 1.0"
        );
    }

    #[test]
    fn fire_clamps_at_zero_when_hp_already_zero() {
        let mut world = World::new();
        let entity = world
            .spawn(Hp {
                current:  0.0,
                starting: 3.0,
                max:      None,
            })
            .id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().current).abs() < f32::EPSILON,
            "Hp should remain clamped at 0.0"
        );
    }

    #[test]
    fn fire_on_entity_without_hp_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Should not panic.
        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            world.get::<Hp>(entity).is_none(),
            "No Hp component should be added"
        );
    }

    #[test]
    fn multiple_fires_decrement_cumulatively() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(5.0)).id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);
        LoseLifeConfig {}.fire(entity, "test_source", &mut world);
        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().current - 2.0).abs() < f32::EPSILON,
            "Hp should be 2.0 after three decrements from 5.0"
        );
    }

    #[test]
    fn fire_does_not_modify_starting_hp() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(5.0)).id();

        LoseLifeConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            (world.get::<Hp>(entity).unwrap().starting - 5.0).abs() < f32::EPSILON,
            "starting Hp should remain unchanged at 5.0"
        );
    }
}
