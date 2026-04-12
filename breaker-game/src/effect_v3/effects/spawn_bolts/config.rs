//! `SpawnBoltsConfig` — fire-and-forget bolt spawning.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Spawns extra bolts at the entity's position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnBoltsConfig {
    /// Number of bolts to spawn.
    pub count:    u32,
    /// Optional duration in seconds before each spawned bolt despawns (None = permanent).
    pub lifespan: Option<OrderedFloat<f32>>,
    /// Whether spawned bolts copy the first primary bolt's effect trees.
    pub inherit:  bool,
}

impl Fireable for SpawnBoltsConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        // TODO: full implementation requires bolt builder integration
        warn!(
            "SpawnBolts::fire() called but bolt builder is not yet integrated (count={})",
            self.count,
        );
    }
}
