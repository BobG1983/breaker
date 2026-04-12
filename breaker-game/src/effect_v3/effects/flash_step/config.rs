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
