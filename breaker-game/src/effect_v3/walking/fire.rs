//! Fire node evaluator — immediate effect execution.

use bevy::prelude::*;

use crate::effect_v3::{
    commands::FireEffectCommand,
    types::{EffectType, TriggerContext},
};

/// Evaluate a `Tree::Fire` node: queue a deferred command to fire the effect
/// on the entity.
pub fn evaluate_fire(
    entity: Entity,
    effect: &EffectType,
    source: &str,
    _context: &TriggerContext,
    commands: &mut Commands,
) {
    commands.queue(FireEffectCommand {
        entity,
        effect: effect.clone(),
        source: source.to_owned(),
    });
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

    #[test]
    fn evaluate_fire_queues_fire_command_that_creates_stack() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let effect = EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        });
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_fire(entity, &effect, "test_chip", &context, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
    }
}
