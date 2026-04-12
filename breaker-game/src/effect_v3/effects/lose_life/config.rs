//! `LoseLifeConfig` — fire-and-forget life decrement.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, shared::death_pipeline::Hp};

/// Decrements the entity's Hp by 1. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoseLifeConfig {}

impl Fireable for LoseLifeConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if let Some(mut hp) = world.get_mut::<Hp>(entity) {
            hp.current = (hp.current - 1.0).max(0.0);
        }
    }
}
