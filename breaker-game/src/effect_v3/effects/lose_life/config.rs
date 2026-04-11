//! `LoseLifeConfig` — fire-and-forget life decrement.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Decrements the entity's Hp by 1. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoseLifeConfig {}

impl Fireable for LoseLifeConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
