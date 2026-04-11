//! Sequence node evaluator — ordered multi-execute.

use bevy::prelude::*;

use crate::effect_v3::{
    commands::{FireEffectCommand, RouteEffectCommand},
    types::{Terminal, TriggerContext},
};

/// Evaluate a `Tree::Sequence` node: run children left to right.
pub fn evaluate_sequence(
    entity: Entity,
    terminals: &[Terminal],
    _context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    for terminal in terminals {
        evaluate_terminal(entity, terminal, source, commands);
    }
}

/// Evaluate a single Terminal node — either fire an effect or route a tree.
pub fn evaluate_terminal(
    entity: Entity,
    terminal: &Terminal,
    source: &str,
    commands: &mut Commands,
) {
    match terminal {
        Terminal::Fire(effect_type) => {
            commands.queue(FireEffectCommand {
                entity,
                effect: effect_type.clone(),
                source: source.to_owned(),
            });
        }
        Terminal::Route(route_type, tree) => {
            commands.queue(RouteEffectCommand {
                entity,
                name: source.to_owned(),
                tree: (**tree).clone(),
                route_type: *route_type,
            });
        }
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
        types::EffectType,
    };

    #[test]
    fn evaluate_sequence_fires_all_terminals_in_order() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let terminals = vec![
            Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
            Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            })),
        ];
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_sequence(entity, &terminals, &context, "test_chip", &mut commands);
        }
        queue.apply(&mut world);

        let speed_stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
        assert_eq!(speed_stack.len(), 1);

        let dmg_stack = world.get::<EffectStack<DamageBoostConfig>>(entity).unwrap();
        assert_eq!(dmg_stack.len(), 1);
    }
}
