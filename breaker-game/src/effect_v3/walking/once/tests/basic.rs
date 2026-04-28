use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, Terminal, Tree, Trigger, TriggerContext},
        walking::walk_effects::walk_bound_effects,
    },
    prelude::DamageBoostStack,
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        walk_bound_effects(
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
        .get::<DamageBoostStack>(entity)
        .expect("DamageBoostStack should exist");
    assert!(!dmg_stack.is_empty());
    assert!(
        (dmg_stack.aggregate_persistent(None) - 2.0).abs() < 1e-5,
        "DamageBoostStack aggregate should be 2.0, got {}",
        dmg_stack.aggregate_persistent(None)
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects after Once fired"
    );
}
