//! `FlashStepConfig` — enable flash step dash on the breaker.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::FlashStepActive;
use crate::effect_v3::traits::{Fireable, Reversible};

/// Enables flash step dash on the breaker. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlashStepConfig {}

impl Fireable for FlashStepConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).insert(FlashStepActive);
        }
    }
}

impl Reversible for FlashStepConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<FlashStepActive>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::effect_v3::{
        effects::flash_step::FlashStepActive,
        traits::{Fireable, Reversible},
    };

    #[test]
    fn reverse_all_by_source_removes_flash_step_active_via_default_delegation() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        FlashStepConfig {}.fire(entity, "dash_chip", &mut world);
        assert!(world.get::<FlashStepActive>(entity).is_some());

        FlashStepConfig {}.reverse_all_by_source(entity, "dash_chip", &mut world);
        assert!(
            world.get::<FlashStepActive>(entity).is_none(),
            "FlashStepActive should be removed by default delegation"
        );

        // Calling twice does not panic.
        FlashStepConfig {}.reverse_all_by_source(entity, "dash_chip", &mut world);
    }
}
