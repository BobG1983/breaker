//! `RampingDamageConfig` — additive passive ramping damage.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::RampingDamageAccumulator;
use crate::effect_v3::{
    stacking::EffectStack,
    traits::{Fireable, PassiveEffect, Reversible},
};

/// Flat damage bonus added per activation — accumulates each time the trigger fires.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RampingDamageConfig {
    /// Flat damage increment per activation.
    pub increment: OrderedFloat<f32>,
}

impl Fireable for RampingDamageConfig {
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
        // Insert accumulator if not already present.
        if world.get::<RampingDamageAccumulator>(entity).is_none() {
            world
                .entity_mut(entity)
                .insert(RampingDamageAccumulator(OrderedFloat(0.0)));
        }
    }

    fn register(app: &mut App) {
        use super::systems::reset_ramping_damage;
        use crate::{effect_v3::EffectV3Systems, state::types::NodeState};

        app.add_systems(
            OnEnter(NodeState::Loading),
            reset_ramping_damage.in_set(EffectV3Systems::Reset),
        );
    }
}

impl Reversible for RampingDamageConfig {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World) {
        if let Some(mut stack) = world.get_mut::<EffectStack<Self>>(entity) {
            stack.remove(source, self);
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
            stack.retain_by_source(source);
            if stack.is_empty() {
                world
                    .entity_mut(entity)
                    .remove::<RampingDamageAccumulator>();
            }
        }
    }
}

impl PassiveEffect for RampingDamageConfig {
    fn aggregate(entries: &[(String, Self)]) -> f32 {
        entries.iter().map(|(_, c)| c.increment.into_inner()).sum()
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
        let config = RampingDamageConfig {
            increment: OrderedFloat(0.5),
        };

        config.fire(entity, "test_source", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn fire_multiple_times_stacks_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = RampingDamageConfig {
            increment: OrderedFloat(0.5),
        };

        config.fire(entity, "test_source", &mut world);
        config.fire(entity, "test_source", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = RampingDamageConfig {
            increment: OrderedFloat(0.5),
        };

        config.fire(entity, "test_source", &mut world);
        config.reverse(entity, "test_source", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn reverse_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let config = RampingDamageConfig {
            increment: OrderedFloat(0.5),
        };

        config.reverse(entity, "test_source", &mut world);
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_all_entries_from_matching_source_leaves_others() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "amp", &mut world);
        RampingDamageConfig {
            increment: OrderedFloat(0.25),
        }
        .fire(entity, "feedback_loop", &mut world);
        RampingDamageConfig {
            increment: OrderedFloat(1.0),
        }
        .fire(entity, "amp", &mut world);

        // Manually set accumulator to a non-zero value.
        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(3.0)));

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse_all_by_source(entity, "amp", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.25).abs() < 1e-5);
        // Accumulator should still be present because the stack is non-empty.
        assert!(
            world.get::<RampingDamageAccumulator>(entity).is_some(),
            "RampingDamageAccumulator should still be present when stack is non-empty"
        );
    }

    #[test]
    fn reverse_all_by_source_removes_accumulator_when_stack_becomes_empty() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "amp", &mut world);
        RampingDamageConfig {
            increment: OrderedFloat(1.0),
        }
        .fire(entity, "amp", &mut world);

        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(2.5)));

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse_all_by_source(entity, "amp", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert!(stack.is_empty());
        assert!(
            world.get::<RampingDamageAccumulator>(entity).is_none(),
            "RampingDamageAccumulator should be removed when stack becomes empty"
        );
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_stack_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse_all_by_source(entity, "amp", &mut world);
        // No panic.
    }
}
