//! `VulnerableConfig` — multiplicative passive incoming damage scaling.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::{
    stacking::EffectStack,
    traits::{Fireable, PassiveEffect, Reversible},
};

/// Incoming damage multiplier — values above 1.0 increase damage taken.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VulnerableConfig {
    /// Incoming damage multiplier.
    pub multiplier: OrderedFloat<f32>,
}

impl Fireable for VulnerableConfig {
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

impl Reversible for VulnerableConfig {
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

impl PassiveEffect for VulnerableConfig {
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
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<VulnerableConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<VulnerableConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 2.25).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.fire(entity, "test_source", &mut world);
        config.reverse(entity, "test_source", &mut world);

        let stack = world.get::<EffectStack<VulnerableConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        };

        config.reverse(entity, "test_source", &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .fire(entity, "decay", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(0.5),
        }
        .fire(entity, "shield_effect", &mut world);
        VulnerableConfig {
            multiplier: OrderedFloat(2.0),
        }
        .fire(entity, "decay", &mut world);

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, "decay", &mut world);

        let stack = world.get::<EffectStack<VulnerableConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        VulnerableConfig {
            multiplier: OrderedFloat(1.5),
        }
        .reverse_all_by_source(entity, "decay", &mut world);
        // No panic.
    }
}
