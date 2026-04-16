use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::effect_v3::{
    effects::SpeedBoostConfig,
    stacking::EffectStack,
    storage::BoundEffects,
    types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
    walking::walk_effects::walk_bound_effects,
};

// ----------------------------------------------------------------
// Behavior 4: Until self-removes from BoundEffects after reversal
// ----------------------------------------------------------------

#[test]
fn until_removes_entry_from_bound_effects_after_reversal() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // First walk: apply
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

    // Second walk: gate trigger matches — should reverse and remove
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

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects after Until reversal"
    );
}

// Edge case for behavior 4: BoundEffects with only Until entry becomes empty vec
#[test]
fn until_bound_effects_becomes_empty_after_sole_entry_reversal() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // Apply
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

    // Reverse
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

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        remaining.is_empty(),
        "BoundEffects should be empty vec after sole Until entry is reversed and removed"
    );
}

// ----------------------------------------------------------------
// Behavior 7: Until does not reverse on non-matching trigger after
//             application
// ----------------------------------------------------------------

#[test]
fn until_does_not_reverse_on_non_matching_trigger_after_application() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // First walk: apply
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

    // Second walk: non-matching trigger — should not reverse
    let trees_second = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::BoltLostOccurred,
            &TriggerContext::None,
            &trees_second,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should still exist after non-matching walk");
    assert_eq!(
        stack.len(),
        1,
        "Effect should still be active after non-matching trigger"
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should still be in BoundEffects after non-matching trigger"
    );
}

// ----------------------------------------------------------------
// Behavior 8: Until produces no additional effects or reversals
//             after self-removal
// ----------------------------------------------------------------

#[test]
fn until_produces_no_effects_after_self_removal() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Until(
            Trigger::Bumped,
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ))),
        ),
    )]);
    world.entity_mut(entity).insert(bound);

    // First walk: apply
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

    // Verify the first walk actually fired the effect (precondition)
    let stack_after_fire = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after first walk fires Until inner effect");
    assert_eq!(
        stack_after_fire.len(),
        1,
        "Precondition: Until must fire inner effect on first walk"
    );

    // Second walk: gate trigger — reverse and remove
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

    // Third walk: walk with gate trigger again — should be a no-op
    let trees_third = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue3 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue3, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees_third,
            &mut commands,
        );
    }
    queue3.apply(&mut world);

    // Fourth walk: non-matching trigger — also no-op
    let trees_fourth = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue4 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue4, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees_fourth,
            &mut commands,
        );
    }
    queue4.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    let is_empty = stack.is_none() || stack.unwrap().is_empty();
    assert!(
        is_empty,
        "No new EffectStack entries should appear after Until self-removes"
    );
}
