//! `VulnerableConfig` — multiplicative passive incoming damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Incoming damage multiplier — values above 1.0 increase damage taken.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VulnerableConfig {
    /// Incoming damage multiplier.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for VulnerableConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for VulnerableConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for VulnerableConfig {
    fn aggregate(_entries: &[(String, Self)]) -> f32 {
        todo!()
    }
}
