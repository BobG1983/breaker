//! `RandomEffectConfig` — fire-and-forget random effect selection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{traits::Fireable, types::EffectType};

/// Picks a random effect from a weighted pool and fires it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RandomEffectConfig {
    /// Weighted list of effects — each entry is (weight, effect). Fires exactly one per activation.
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for RandomEffectConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
