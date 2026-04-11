//! `EntropyConfig` — random effect trigger based on bump accumulation.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{
    traits::{Fireable, Reversible},
    types::EffectType,
};

/// Configuration for the entropy engine mechanic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntropyConfig {
    /// Cap on how many effects fire per activation.
    pub max_effects: u32,
    /// Weighted list of effects — each entry is (weight, effect).
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for EntropyConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for EntropyConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
