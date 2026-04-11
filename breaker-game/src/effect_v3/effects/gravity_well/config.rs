//! `GravityWellConfig` — gravity well pulling bolts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Configuration for a point attractor that pulls bolts toward it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GravityWellConfig {
    /// How strongly bolts are pulled toward the well center per tick.
    pub strength: OrderedFloat<f32>,
    /// How long the well exists before despawning.
    pub duration: OrderedFloat<f32>,
    /// How far from the well center bolts are affected.
    pub radius: OrderedFloat<f32>,
    /// Maximum active wells per owner entity — oldest removed when exceeded.
    pub max: u32,
}

impl Fireable for GravityWellConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
