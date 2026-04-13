//! `EntropyConfig` — random effect trigger based on bump accumulation.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::EntropyCounter;
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
    pub pool:        Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}

impl Fireable for EntropyConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert(EntropyCounter {
            count:       0,
            max_effects: self.max_effects,
            pool:        self.pool.clone(),
        });
    }

    fn register(app: &mut App) {
        use super::systems::{reset_entropy_counter, tick_entropy_engine};
        use crate::{effect_v3::EffectV3Systems, state::types::NodeState};

        app.add_systems(
            OnEnter(NodeState::Loading),
            reset_entropy_counter.in_set(EffectV3Systems::Reset),
        );
        app.add_systems(
            FixedUpdate,
            tick_entropy_engine.in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for EntropyConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<EntropyCounter>();
        }
    }
}
