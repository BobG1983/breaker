//! `SpawnPhantomConfig` — spawn phantom bolt with limited lifetime.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Configuration for spawning a temporary phantom bolt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnPhantomConfig {
    /// How long the phantom bolt exists before despawning.
    pub duration:   OrderedFloat<f32>,
    /// Maximum phantom bolts from this source that can exist at once.
    pub max_active: u32,
}

impl Fireable for SpawnPhantomConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        // TODO: full implementation requires bolt builder integration
        warn!(
            "SpawnPhantom::fire() called but bolt builder is not yet integrated (duration={}, max_active={})",
            self.duration, self.max_active,
        );
    }

    fn register(app: &mut App) {
        use super::systems::tick_phantom_lifetime;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_phantom_lifetime.in_set(EffectV3Systems::Tick),
        );
    }
}
