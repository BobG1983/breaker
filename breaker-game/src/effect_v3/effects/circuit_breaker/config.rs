//! `CircuitBreakerConfig` — bump counter toward automatic shockwave.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, Reversible};

/// Configuration for the circuit breaker counter mechanic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Bumps needed per cycle before the reward fires.
    pub bumps_required: u32,
    /// Number of extra bolts spawned as the reward.
    pub spawn_count: u32,
    /// Whether spawned bolts inherit effect trees.
    pub inherit: bool,
    /// Maximum radius of the reward shockwave.
    pub shockwave_range: OrderedFloat<f32>,
    /// Expansion speed of the reward shockwave.
    pub shockwave_speed: OrderedFloat<f32>,
}

impl Fireable for CircuitBreakerConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for CircuitBreakerConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
