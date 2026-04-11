//! `ShockwaveConfig` — expanding damage shockwave.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Configuration for an expanding radial shockwave that damages cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShockwaveConfig {
    /// How far the shockwave ring expands before disappearing.
    pub base_range: OrderedFloat<f32>,
    /// Extra range added per stack beyond the first.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count — effective range is `base_range` + `range_per_level` * (stacks - 1).
    pub stacks: u32,
    /// How fast the ring expands outward in world units per second.
    pub speed: OrderedFloat<f32>,
}

impl Fireable for ShockwaveConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
