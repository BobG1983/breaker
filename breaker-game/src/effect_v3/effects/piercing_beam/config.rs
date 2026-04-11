//! `PiercingBeamConfig` — fire-and-forget piercing beam line.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Fires a beam that damages all cells along a line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PiercingBeamConfig {
    /// Multiplier applied to base damage for cells hit by the beam.
    pub damage_mult: OrderedFloat<f32>,
    /// Width of the beam rectangle in world units.
    pub width: OrderedFloat<f32>,
}

impl Fireable for PiercingBeamConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
