//! `AttractionConfig` — attraction steering toward entities.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::AttractionType,
};

/// Configuration for bolt attraction toward a target entity type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttractionConfig {
    /// Which entity type the bolt steers toward.
    pub attraction_type: AttractionType,
    /// Attraction strength per tick.
    pub force: OrderedFloat<f32>,
    /// Optional cap on the per-tick steering delta (None = uncapped).
    pub max_force: Option<OrderedFloat<f32>>,
}

impl Fireable for AttractionConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for AttractionConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
