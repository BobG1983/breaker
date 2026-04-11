//! `ChainBoltConfig` — fire-and-forget chain bolt redirect.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Tethers two bolts with a distance constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainBoltConfig {
    /// Maximum distance between the two tethered bolts before the constraint pulls them back.
    pub tether_distance: OrderedFloat<f32>,
}

impl Fireable for ChainBoltConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
