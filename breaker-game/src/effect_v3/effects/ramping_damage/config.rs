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

    // ── fire inserts accumulator ──────────────────────────────────────

    #[test]
    fn fire_inserts_accumulator_if_absent() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "amp", &mut world);

        let acc = world.get::<RampingDamageAccumulator>(entity);
        assert!(acc.is_some(), "fire should insert RampingDamageAccumulator");
        assert_eq!(
            acc.unwrap().0,
            OrderedFloat(0.0),
            "newly inserted accumulator should be 0.0",
        );
    }

    #[test]
    fn fire_does_not_overwrite_existing_accumulator() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Fire once to create the stack + accumulator.
        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "amp", &mut world);

        // Manually set accumulator to non-zero to simulate gameplay usage.
        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(3.5)));

        // Fire again from a different source.
        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "feedback_loop", &mut world);

        let acc = world.get::<RampingDamageAccumulator>(entity).unwrap();
        assert_eq!(
            acc.0,
            OrderedFloat(3.5),
            "fire should not overwrite existing accumulator value",
        );

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(
            stack.len(),
            2,
            "stack should have 2 entries after second fire"
        );
    }

    // ── reverse removes accumulator when stack becomes empty ──────────

    #[test]
    fn reverse_with_single_entry_removes_accumulator() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .fire(entity, "amp", &mut world);

        // Manually set accumulator to non-zero.
        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(2.0)));

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse(entity, "amp", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert!(stack.is_empty(), "stack should be empty after reverse");
        assert!(
            world.get::<RampingDamageAccumulator>(entity).is_none(),
            "accumulator should be removed when stack becomes empty",
        );
    }

    #[test]
    fn reverse_on_entity_without_accumulator_does_not_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Manually insert stack with one entry but no accumulator.
        world
            .entity_mut(entity)
            .insert(EffectStack::<RampingDamageConfig>::default());
        world
            .get_mut::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap()
            .push(
                "amp".to_owned(),
                RampingDamageConfig {
                    increment: OrderedFloat(0.5),
                },
            );

        // Should not panic even without accumulator component.
        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse(entity, "amp", &mut world);
    }

    // ── reverse with non-empty stack keeps accumulator ────────────────

    #[test]
    fn reverse_with_non_empty_stack_keeps_accumulator() {
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

        // Set accumulator to 1.5.
        world
            .entity_mut(entity)
            .insert(RampingDamageAccumulator(OrderedFloat(1.5)));

        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse(entity, "amp", &mut world);

        let stack = world
            .get::<EffectStack<RampingDamageConfig>>(entity)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.25).abs() < 1e-5);

        let acc = world.get::<RampingDamageAccumulator>(entity);
        assert!(
            acc.is_some(),
            "accumulator should remain when stack is non-empty",
        );
        assert_eq!(
            acc.unwrap().0,
            OrderedFloat(1.5),
            "accumulator value should be preserved",
        );
    }

    #[test]
    fn reverse_with_non_empty_stack_keeps_zero_accumulator() {
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

        // Accumulator is already 0.0 from fire.
        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        }
        .reverse(entity, "amp", &mut world);

        let acc = world.get::<RampingDamageAccumulator>(entity);
        assert!(
            acc.is_some(),
            "zero-valued accumulator should still be kept when stack is non-empty",
        );
        assert_eq!(acc.unwrap().0, OrderedFloat(0.0));
    }
}
