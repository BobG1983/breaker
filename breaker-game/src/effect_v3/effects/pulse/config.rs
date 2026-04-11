//! `PulseConfig` — periodic shockwave emitter.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for periodic pulse shockwave emission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PulseConfig {
    /// Radius of each pulse shockwave.
    pub base_range: OrderedFloat<f32>,
    /// Extra range per stack.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count.
    pub stacks: u32,
    /// Expansion speed of each pulse ring.
    pub speed: OrderedFloat<f32>,
    /// Seconds between each pulse emission.
    pub interval: OrderedFloat<f32>,
}

impl Fireable for PulseConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for PulseConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
