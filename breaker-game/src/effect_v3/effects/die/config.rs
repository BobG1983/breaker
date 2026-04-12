//! `DieConfig` — fire-and-forget entity death.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, shared::death_pipeline::Dead};

/// Sends the entity into the death pipeline. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DieConfig {}

impl Fireable for DieConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        // Mark entity as Dead — the death pipeline will process it
        if world.get_entity(entity).is_ok() && world.get::<Dead>(entity).is_none() {
            world.entity_mut(entity).insert(Dead);
        }
    }
}
