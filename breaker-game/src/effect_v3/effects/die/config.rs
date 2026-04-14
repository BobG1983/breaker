//! `DieConfig` — fire-and-forget entity death.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, shared::death_pipeline::Dead};

/// Sends the entity into the death pipeline. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DieConfig {}

impl Fireable for DieConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        // Mark entity as Dead — the death pipeline will process it
        if world.get_entity(entity).is_ok() && world.get::<Dead>(entity).is_none() {
            world.entity_mut(entity).insert(Dead);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::{
        effect_v3::traits::Fireable,
        shared::death_pipeline::{Dead, Hp},
    };

    #[test]
    fn fire_inserts_dead_on_living_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        DieConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            world.get::<Dead>(entity).is_some(),
            "Dead component should be inserted on a living entity"
        );
    }

    #[test]
    fn fire_inserts_dead_without_removing_other_components() {
        let mut world = World::new();
        let entity = world.spawn(Hp::new(5.0)).id();

        DieConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            world.get::<Dead>(entity).is_some(),
            "Dead should be inserted"
        );
        assert!(
            (world.get::<Hp>(entity).unwrap().current - 5.0).abs() < f32::EPSILON,
            "Hp should be untouched"
        );
    }

    #[test]
    fn fire_on_already_dead_entity_is_idempotent() {
        let mut world = World::new();
        let entity = world.spawn(Dead).id();

        DieConfig {}.fire(entity, "test_source", &mut world);

        assert!(
            world.get::<Dead>(entity).is_some(),
            "Dead should still be present after idempotent fire"
        );
    }

    #[test]
    fn fire_on_despawned_entity_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        world.despawn(entity);

        // Should not panic.
        DieConfig {}.fire(entity, "test_source", &mut world);
    }
}
