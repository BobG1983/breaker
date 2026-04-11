//! `ExplodeConfig` — fire-and-forget area explosion.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Area explosion dealing flat damage to all cells within range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExplodeConfig {
    /// Radius of the explosion in world units.
    pub range: OrderedFloat<f32>,
    /// Flat damage dealt to every cell within range.
    pub damage: OrderedFloat<f32>,
}

impl Fireable for ExplodeConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
