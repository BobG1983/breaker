//! `RandomEffectConfig` — fire-and-forget random effect selection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    effect_v3::{dispatch::fire_dispatch, traits::Fireable, types::EffectType},
    shared::rng::GameRng,
};

/// Picks a random effect from a weighted pool and fires it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RandomEffectConfig {
    /// Weighted list of effects — each entry is (weight, effect). Fires exactly one per activation.
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for RandomEffectConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if self.pool.is_empty() {
            return;
        }

        let total_weight: f32 = self.pool.iter().map(|(w, _)| w.0).sum();
        if total_weight <= 0.0 {
            return;
        }

        // Pick a weighted random effect from the pool.
        let roll: f32 = world.resource_mut::<GameRng>().0.random::<f32>() * total_weight;
        let mut accumulated = 0.0;
        let mut chosen: Option<&EffectType> = None;
        for (weight, effect) in &self.pool {
            accumulated += weight.0;
            if roll < accumulated {
                chosen = Some(effect);
                break;
            }
        }

        // Fallback to last entry if floating-point imprecision causes no match.
        let Some(fallback) = self.pool.last() else {
            return;
        };
        let effect = chosen.unwrap_or(&fallback.1);
        fire_dispatch(effect, entity, source, world);
    }
}
