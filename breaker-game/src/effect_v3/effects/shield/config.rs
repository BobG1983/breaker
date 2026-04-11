//! `ShieldConfig` — shield wall protection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for a temporary shield wall that reflects bolts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// How long the shield wall lasts in seconds.
    pub duration: OrderedFloat<f32>,
    /// Seconds subtracted from the shield's remaining time each time a bolt bounces off it.
    pub reflection_cost: OrderedFloat<f32>,
}

impl Fireable for ShieldConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for ShieldConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
