//! `SpeedBoostConfig` — multiplicative passive speed scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Multiplicative speed scaling factor applied to the entity's base speed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeedBoostConfig {
    /// Multiplicative speed scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for SpeedBoostConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for SpeedBoostConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for SpeedBoostConfig {
    fn aggregate(_entries: &[(String, Self)]) -> f32 {
        todo!()
    }
}
