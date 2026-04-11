//! `RampingDamageConfig` — additive passive ramping damage.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Flat damage bonus added per activation — accumulates each time the trigger fires.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RampingDamageConfig {
    /// Flat damage increment per activation.
    pub increment: OrderedFloat<f32>,
}

impl Fireable for RampingDamageConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for RampingDamageConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for RampingDamageConfig {
    fn aggregate(_entries: &[(String, Self)]) -> f32 {
        todo!()
    }
}
