//! `DamageBoostConfig` — multiplicative passive damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Multiplicative damage scaling factor applied to the entity's base damage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageBoostConfig {
    /// Multiplicative damage scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for DamageBoostConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for DamageBoostConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for DamageBoostConfig {
    fn aggregate(_entries: &[(String, Self)]) -> f32 {
        todo!()
    }
}
