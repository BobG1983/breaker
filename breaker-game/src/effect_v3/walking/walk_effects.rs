//! `walk_effects` — outer loop for evaluating effect trees against a trigger.

use bevy::prelude::*;

use super::{
    evaluate_during, evaluate_fire, evaluate_on, evaluate_once, evaluate_sequence, evaluate_until,
    evaluate_when,
};
use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Walk all effect trees on an entity, evaluating nodes against the given
/// trigger and context.
///
/// This is the main entry point for trigger dispatch. Bridge systems call
/// this after building a `TriggerContext` from a game event.
pub fn walk_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut Commands,
) {
    for (source, tree) in trees {
        evaluate_tree(entity, tree, trigger, context, source, commands);
    }
}

/// Recursively evaluate a single tree node against the active trigger.
pub fn evaluate_tree(
    entity: Entity,
    tree: &Tree,
    trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    match tree {
        Tree::Fire(effect_type) => {
            evaluate_fire(entity, effect_type, source, context, commands);
        }
        Tree::When(gate_trigger, inner) => {
            evaluate_when(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::Once(gate_trigger, inner) => {
            evaluate_once(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::During(condition, inner) => {
            evaluate_during(entity, condition, inner, context, source, commands);
        }
        Tree::Until(gate_trigger, inner) => {
            evaluate_until(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::Sequence(terminals) => {
            evaluate_sequence(entity, terminals, context, source, commands);
        }
        Tree::On(target, terminal) => {
            evaluate_on(entity, *target, terminal, context, source, commands);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack, types::EffectType};

    #[test]
    fn walk_effects_fire_tree_queues_fire_command() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        )];

        let trigger = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after walk_effects fires a SpeedBoost");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn walk_effects_when_matching_trigger_fires_inner() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )];

        let trigger = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after matching When trigger");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn walk_effects_when_non_matching_trigger_does_nothing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )];

        // Use a different trigger that doesn't match
        let trigger = Trigger::BoltLostOccurred;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "No EffectStack should exist when trigger doesn't match"
        );
    }
}
