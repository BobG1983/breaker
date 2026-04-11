//! `ChainLightningConfig` — chain lightning arcs between cells.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Configuration for chain lightning that arcs between nearby cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainLightningConfig {
    /// Number of times the lightning jumps between cells.
    pub arcs: u32,
    /// Maximum distance each arc can jump to find a new target.
    pub range: OrderedFloat<f32>,
    /// Multiplier applied to base damage for each arc hit.
    pub damage_mult: OrderedFloat<f32>,
    /// How fast each lightning arc travels between cells in world units per second.
    pub arc_speed: OrderedFloat<f32>,
}

impl Fireable for ChainLightningConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
