//! Once node evaluator — one-shot trigger gate.

use bevy::prelude::*;

use super::walk_effects::evaluate_tree;
use crate::effect_v3::{
    commands::RemoveEffectCommand,
    types::{Tree, Trigger, TriggerContext},
};

/// Evaluate a `Tree::Once` node: if the trigger matches, evaluate the inner tree
/// and then remove this node so it never fires again.
pub fn evaluate_once(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &Tree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if gate_trigger == active_trigger {
        evaluate_tree(entity, inner, active_trigger, context, source, commands);
        commands.queue(RemoveEffectCommand {
            entity,
            name: source.to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, Terminal},
        walking::walk_effects::walk_effects,
    };

    // ----- Behavior 1: Once fires inner tree on first matching trigger -----

    #[test]
    fn once_fires_inner_tree_on_matching_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after Once fires on matching trigger");
        assert_eq!(stack.len(), 1);
    }

    // ----- Behavior 2: Once queues RemoveEffectCommand that removes entry from BoundEffects -----

    #[test]
    fn once_removes_entry_from_bound_effects_after_firing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should have been removed from BoundEffects after Once fired"
        );
    }

    // ----- Behavior 3: Once does not fire on non-matching trigger -----

    #[test]
    fn once_does_not_fire_on_non_matching_trigger() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "No EffectStack should exist when trigger doesn't match"
        );

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should still be in BoundEffects when trigger doesn't match"
        );
    }

    // ----- Behavior 4: Once does not fire a second time after removal -----

    #[test]
    fn once_does_not_fire_second_time_after_removal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )]);
        world.entity_mut(entity).insert(bound);

        // First walk: fires and should remove chip_a
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // BoundEffects should be empty after removal
        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            remaining.is_empty(),
            "BoundEffects should be empty after Once fired and removed chip_a"
        );

        // Second walk: re-read BoundEffects (now empty), walk again
        let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue2 = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue2, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees_second,
                &mut commands,
            );
        }
        queue2.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist from the first firing");
        assert_eq!(
            stack.len(),
            1,
            "Stack should have exactly 1 entry (not 2) — Once must not fire a second time"
        );
    }

    // ----- Behavior 5: Once removal does not affect other entries in BoundEffects -----

    #[test]
    fn once_removal_does_not_affect_other_bound_effects_entries() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![
            (
                "chip_a".to_string(),
                Tree::Once(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            ),
            (
                "chip_b".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            ),
        ]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // Both should have fired
        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after both trees fire");
        assert_eq!(stack.len(), 2, "Both Once and When should fire");

        // Only chip_a (Once) should be removed, chip_b (When) stays
        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert_eq!(
            remaining.len(),
            1,
            "Only chip_b should remain in BoundEffects"
        );
        assert_eq!(remaining[0].0, "chip_b");
    }

    // ----- Behavior 6: Once with nested Sequence fires all terminals before removal -----

    #[test]
    fn once_with_nested_sequence_fires_all_terminals_before_removal() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Sequence(vec![
                    Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    })),
                    Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    })),
                ])),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let speed_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("SpeedBoost EffectStack should exist");
        assert_eq!(speed_stack.len(), 1);

        let dmg_stack = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .expect("DamageBoost EffectStack should exist");
        assert_eq!(dmg_stack.len(), 1);

        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects after Once fired"
        );
    }

    // ----- Edge case (reviewer): Once removes on trigger match even if inner tree produces no effect -----

    #[test]
    fn once_removes_on_trigger_match_even_when_inner_tree_produces_no_effect() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Inner tree is a When(Died, ...) which won't match Bumped trigger
        // So the inner tree produces no effect, but Once should still remove itself
        let bound = BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )]);
        world.entity_mut(entity).insert(bound);
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        // No effect should fire (inner When(Died) didn't match Bumped)
        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "No effect should fire because inner When(Died) didn't match Bumped"
        );

        // But the Once entry should still be removed because the outer trigger matched
        let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
        assert!(
            !remaining.iter().any(|(name, _)| name == "chip_a"),
            "chip_a should be removed from BoundEffects even though inner tree produced no effect"
        );
    }
}
