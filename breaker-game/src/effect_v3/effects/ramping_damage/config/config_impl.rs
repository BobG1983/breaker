//! `RampingDamageConfig` — additive passive ramping damage.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::super::components::RampingDamageAccumulator;
use crate::{
    effect_v3::{
        stacking::EffectStack,
        traits::{Fireable, PassiveEffect, Reversible},
    },
    prelude::SourceId,
};

/// Flat damage bonus added per activation — accumulates each time the trigger fires.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RampingDamageConfig {
    /// Flat damage increment per activation.
    pub increment: OrderedFloat<f32>,
}

impl Fireable for RampingDamageConfig {
    fn fire(
        &self,
        entity: Entity,
        source: &str,
        world: &mut World,
        _rng: &mut rand_chacha::ChaCha8Rng,
    ) {
        let has_stack = world.get::<EffectStack<Self>>(entity).is_some();
        if !has_stack {
            world
                .entity_mut(entity)
                .insert(EffectStack::<Self>::default());
        }
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.push(source_id, self.clone());
        }
        // Insert accumulator if not already present.
        if world.get::<RampingDamageAccumulator>(entity).is_none() {
            world
                .entity_mut(entity)
                .insert(RampingDamageAccumulator(OrderedFloat(0.0)));
        }
    }

    fn register(app: &mut App) {
        use super::super::systems::reset_ramping_damage;
        use crate::{effect_v3::EffectV3Systems, prelude::*};

        app.add_systems(
            OnEnter(NodeState::Loading),
            reset_ramping_damage.in_set(EffectV3Systems::Reset),
        );
    }
}

impl Reversible for RampingDamageConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.remove(&source_id, self);
            // Remove accumulator when stack is empty.
            if stack.is_empty() {
                world
                    .entity_mut(entity)
                    .remove::<RampingDamageAccumulator>();
            }
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            let source_id = SourceId::from(source.to_owned());
            stack.retain_by_source(&source_id);
            if stack.is_empty() {
                world
                    .entity_mut(entity)
                    .remove::<RampingDamageAccumulator>();
            }
        }
    }
}

impl PassiveEffect for RampingDamageConfig {
    fn aggregate(entries: &[(SourceId, Self)]) -> f32 {
        entries.iter().map(|(_, c)| c.increment.into_inner()).sum()
    }
}
