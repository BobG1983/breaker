//! When node evaluator — repeating trigger gate.

use bevy::prelude::*;

use super::walk_effects::evaluate_tree;
use crate::effect_v3::types::{Tree, Trigger, TriggerContext};

/// Evaluate a `Tree::When` node: if the trigger matches, evaluate the inner tree.
/// Repeats on every match.
pub fn evaluate_when(
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
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack, types::EffectType};

    #[test]
    fn evaluate_when_matching_trigger_fires_inner() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let gate = Trigger::Bumped;
        let active = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &gate,
                &inner,
                &active,
                &context,
                "test_chip",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn evaluate_when_non_matching_trigger_does_nothing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let gate = Trigger::Bumped;
        let active = Trigger::BoltLostOccurred;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_when(
                entity,
                &gate,
                &inner,
                &active,
                &context,
                "test_chip",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(stack.is_none());
    }
}
