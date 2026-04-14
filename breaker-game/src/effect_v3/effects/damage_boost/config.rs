//! `DamageBoostConfig` — multiplicative passive damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{
    stacking::EffectStack,
    traits::{Fireable, PassiveEffect, Reversible},
};

/// Multiplicative damage scaling factor applied to the entity's base damage.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DamageBoostConfig {
    /// Multiplicative damage scaling factor.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for DamageBoostConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let has_stack = world.get::<EffectStack<Self>>(entity).is_some();
        if !has_stack {
            world
                .entity_mut(entity)
                .insert(EffectStack::<Self>::default());
        }
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.push(source.to_owned(), self.clone());
        }
    }
}

impl Reversible for DamageBoostConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.remove(source, self);
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.retain_by_source(source);
        }
    }
}

impl PassiveEffect for DamageBoostConfig {
    fn aggregate(entries: &[(String, Self)]) -> f32 {
        entries
            .iter()
            .map(|(_, c)| c.multiplier.into_inner())
            .product::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        stacking::EffectStack,
        traits::{Fireable, Reversible},
    };

    #[test]
    fn fire_creates_stack_and_pushes_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<DamageBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<DamageBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.fire(entity, "test_source", &mut world);
        config.reverse(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<DamageBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };

        config.reverse(entity, "test_source", &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "amp", &mut world);
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "feedback_loop", &mut world);
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "amp", &mut world);

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, "amp", &mut world);

        let stack = world.get::<EffectStack<DamageBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 1.5).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }
        .reverse_all_by_source(entity, "amp", &mut world);
        // No panic.
    }
}
