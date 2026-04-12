//! On node evaluator — participant redirection.

use bevy::prelude::*;

use super::sequence::evaluate_terminal;
use crate::effect_v3::types::{
    BoltLostTarget, BumpTarget, DeathTarget, ImpactTarget, ParticipantTarget, Terminal,
    TriggerContext,
};

/// Evaluate a `Tree::On` node: redirect the terminal to the entity
/// identified by the participant target in the trigger context.
pub fn evaluate_on(
    _entity: Entity,
    target: ParticipantTarget,
    terminal: &Terminal,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if let Some(resolved) = resolve_participant(target, context) {
        evaluate_terminal(resolved, terminal, source, commands);
    }
}

const fn resolve_participant(
    target: ParticipantTarget,
    context: &TriggerContext,
) -> Option<Entity> {
    match (target, context) {
        (ParticipantTarget::Bump(BumpTarget::Bolt), TriggerContext::Bump { bolt, .. }) => *bolt,
        (ParticipantTarget::Bump(BumpTarget::Breaker), TriggerContext::Bump { breaker, .. }) => {
            Some(*breaker)
        }
        (
            ParticipantTarget::Impact(ImpactTarget::Impactor),
            TriggerContext::Impact { impactor, .. },
        ) => Some(*impactor),
        (
            ParticipantTarget::Impact(ImpactTarget::Impactee),
            TriggerContext::Impact { impactee, .. },
        ) => Some(*impactee),
        (ParticipantTarget::Death(DeathTarget::Victim), TriggerContext::Death { victim, .. }) => {
            Some(*victim)
        }
        (ParticipantTarget::Death(DeathTarget::Killer), TriggerContext::Death { killer, .. }) => {
            *killer
        }
        (
            ParticipantTarget::BoltLost(BoltLostTarget::Bolt),
            TriggerContext::BoltLost { bolt, .. },
        ) => Some(*bolt),
        (
            ParticipantTarget::BoltLost(BoltLostTarget::Breaker),
            TriggerContext::BoltLost { breaker, .. },
        ) => Some(*breaker),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        types::{BoltLostTarget, BumpTarget, DeathTarget, EffectType, ImpactTarget},
    };

    // ----- Behavior 7: On resolves Bump(Bolt) from Bump context -----

    #[test]
    fn on_resolves_bump_bolt_and_fires_on_bolt_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt_entity = world.spawn_empty().id();

        let context = TriggerContext::Bump {
            bolt:    Some(bolt_entity),
            breaker: owner,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Bump(BumpTarget::Bolt),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let bolt_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("EffectStack should exist on bolt entity");
        assert_eq!(bolt_stack.len(), 1);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(owner_stack.is_none(), "Effect should go to bolt, not owner");
    }

    // ----- Behavior 8: On resolves Bump(Breaker) from Bump context -----

    #[test]
    fn on_resolves_bump_breaker_and_fires_on_breaker_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt_entity = world.spawn_empty().id();
        let breaker_entity = world.spawn_empty().id();

        let context = TriggerContext::Bump {
            bolt:    Some(bolt_entity),
            breaker: breaker_entity,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Bump(BumpTarget::Breaker),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let breaker_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("EffectStack should exist on breaker entity");
        assert_eq!(breaker_stack.len(), 1);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "Effect should go to breaker, not owner"
        );
    }

    // ----- Behavior 9: On resolves Impact(Impactor) from Impact context -----

    #[test]
    fn on_resolves_impact_impactor_and_fires_on_impactor_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let impactor_entity = world.spawn_empty().id();

        let context = TriggerContext::Impact {
            impactor: impactor_entity,
            impactee: owner,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Impact(ImpactTarget::Impactor),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let impactor_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(impactor_entity)
            .expect("EffectStack should exist on impactor entity");
        assert_eq!(impactor_stack.len(), 1);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "Effect should go to impactor, not owner"
        );
    }

    // ----- Behavior 10: On resolves Impact(Impactee) from Impact context -----

    #[test]
    fn on_resolves_impact_impactee_and_fires_on_impactee_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let impactee_entity = world.spawn_empty().id();

        let context = TriggerContext::Impact {
            impactor: owner,
            impactee: impactee_entity,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Impact(ImpactTarget::Impactee),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let impactee_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(impactee_entity)
            .expect("EffectStack should exist on impactee entity");
        assert_eq!(impactee_stack.len(), 1);
    }

    // ----- Behavior 11: On resolves Death(Victim) from Death context -----

    #[test]
    fn on_resolves_death_victim_and_fires_on_victim_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let victim_entity = world.spawn_empty().id();

        let context = TriggerContext::Death {
            victim: victim_entity,
            killer: Some(owner),
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Death(DeathTarget::Victim),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let victim_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(victim_entity)
            .expect("EffectStack should exist on victim entity");
        assert_eq!(victim_stack.len(), 1);
    }

    // ----- Behavior 12: On resolves Death(Killer) from Death context when killer is Some -----

    #[test]
    fn on_resolves_death_killer_and_fires_on_killer_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let killer_entity = world.spawn_empty().id();

        let context = TriggerContext::Death {
            victim: owner,
            killer: Some(killer_entity),
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Death(DeathTarget::Killer),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let killer_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(killer_entity)
            .expect("EffectStack should exist on killer entity");
        assert_eq!(killer_stack.len(), 1);
    }

    // ----- Behavior 13: On resolves BoltLost(Bolt) from BoltLost context -----

    #[test]
    fn on_resolves_bolt_lost_bolt_and_fires_on_bolt_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt_entity = world.spawn_empty().id();

        let context = TriggerContext::BoltLost {
            bolt:    bolt_entity,
            breaker: owner,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::BoltLost(BoltLostTarget::Bolt),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let bolt_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("EffectStack should exist on bolt entity");
        assert_eq!(bolt_stack.len(), 1);
    }

    // ----- Behavior 14: On resolves BoltLost(Breaker) from BoltLost context -----

    #[test]
    fn on_resolves_bolt_lost_breaker_and_fires_on_breaker_entity() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let breaker_entity = world.spawn_empty().id();

        let context = TriggerContext::BoltLost {
            bolt:    owner,
            breaker: breaker_entity,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::BoltLost(BoltLostTarget::Breaker),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let breaker_stack = world
            .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
            .expect("EffectStack should exist on breaker entity");
        assert_eq!(breaker_stack.len(), 1);
    }

    // ----- Behavior 15: On skips when context type does not match participant type -----

    #[test]
    fn on_skips_when_context_type_does_not_match_participant_type() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let bolt_entity = world.spawn_empty().id();

        // Bump context but Impact participant target
        let context = TriggerContext::Bump {
            bolt:    Some(bolt_entity),
            breaker: owner,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Impact(ImpactTarget::Impactor),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "No effect on owner for mismatched context"
        );

        let bolt_stack = world.get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(
            bolt_stack.is_none(),
            "No effect on bolt for mismatched context"
        );
    }

    // ----- Behavior 16: On skips when Bump(Bolt) but bolt is None -----

    #[test]
    fn on_skips_when_bump_bolt_but_bolt_is_none_in_context() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        let context = TriggerContext::Bump {
            bolt:    None,
            breaker: owner,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Bump(BumpTarget::Bolt),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "No effect when bolt is None in Bump context"
        );
    }

    // ----- Behavior 17: On skips when Death(Killer) but killer is None -----

    #[test]
    fn on_skips_when_death_killer_but_killer_is_none_in_context() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        let context = TriggerContext::Death {
            victim: owner,
            killer: None,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Death(DeathTarget::Killer),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "No effect when killer is None in Death context"
        );
    }

    // ----- Behavior 18: On skips when context is TriggerContext::None -----

    #[test]
    fn on_skips_when_context_is_none() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        let context = TriggerContext::None;
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                owner,
                ParticipantTarget::Bump(BumpTarget::Bolt),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let owner_stack = world.get::<EffectStack<SpeedBoostConfig>>(owner);
        assert!(
            owner_stack.is_none(),
            "No effect when context is TriggerContext::None"
        );
    }

    // ----- Behavior 19: On fires on resolved entity even when it is the owner -----

    #[test]
    fn on_fires_on_resolved_entity_even_when_resolved_is_owner() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Entity is both bolt and breaker (degenerate but valid)
        let context = TriggerContext::Bump {
            bolt:    Some(entity),
            breaker: entity,
        };
        let terminal = Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            evaluate_on(
                entity,
                ParticipantTarget::Bump(BumpTarget::Bolt),
                &terminal,
                &context,
                "chip_a",
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist when resolved entity is the owner");
        assert_eq!(stack.len(), 1);
    }
}
