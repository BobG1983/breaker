//! `FlashStepConfig` — enable flash step dash on the breaker.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, Reversible};

/// Enables flash step dash on the breaker. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlashStepConfig {}

impl Fireable for FlashStepConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for FlashStepConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
