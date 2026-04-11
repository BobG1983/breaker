//! `BumpForceConfig` — multiplicative passive bump force scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Multiplicative bump force scaling factor applied to the breaker's bump force.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BumpForceConfig {
    /// Multiplicative bump force scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for BumpForceConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for BumpForceConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for BumpForceConfig {
    fn aggregate(_entries: &[(String, Self)]) -> f32 {
        todo!()
    }
}
