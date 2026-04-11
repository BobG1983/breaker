//! `AnchorConfig` — anchor bolt in place with enhanced bump.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for the anchor effect on the breaker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnchorConfig {
    /// How much the bump force is multiplied when planted.
    pub bump_force_multiplier: OrderedFloat<f32>,
    /// How much wider the perfect timing window becomes when planted.
    pub perfect_window_multiplier: OrderedFloat<f32>,
    /// Seconds the breaker must stand still before planting.
    pub plant_delay: OrderedFloat<f32>,
}

impl Fireable for AnchorConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for AnchorConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
