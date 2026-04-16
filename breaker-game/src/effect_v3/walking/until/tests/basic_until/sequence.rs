use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::effect_v3::{
    effects::{DamageBoostConfig, SpeedBoostConfig},
    stacking::EffectStack,
    storage::BoundEffects,
    types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
    walking::walk_effects::walk_bound_effects,
};

// ----------------------------------------------------------------
// Behavior 5: Until with ScopedTree::Sequence fires all reversible
//             effects on first walk
// ----------------------------------------------------------------

#[test]
fn until_with_sequence_fires_all_reversible_effects_on_first_walk() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Sequence(vec![
                ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }),
                ReversibleEffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                }),
            ])),
        ),
    )]);
    world.entity_mut(entity).insert(bound);
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
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
}

// ----------------------------------------------------------------
// Behavior 6: Until with ScopedTree::Sequence reverses ALL effects
//             on gate trigger match
// ----------------------------------------------------------------

#[test]
fn until_with_sequence_reverses_all_effects_on_gate_trigger_match() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Sequence(vec![
                ReversibleEffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }),
                ReversibleEffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                }),
            ])),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // First walk: apply all
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    // Second walk: gate trigger — reverse all
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees_second,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let speed_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    let dmg_empty = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        speed_empty,
        "SpeedBoost EffectStack should be empty after Sequence reversal"
    );
    assert!(
        dmg_empty,
        "DamageBoost EffectStack should be empty after Sequence reversal"
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects after Sequence reversal"
    );
}
