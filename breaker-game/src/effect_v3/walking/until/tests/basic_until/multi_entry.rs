use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::effect_v3::{
    effects::{DamageBoostConfig, SpeedBoostConfig},
    stacking::EffectStack,
    storage::BoundEffects,
    types::{EffectType, ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
    walking::walk_effects::walk_bound_effects,
};

// ----------------------------------------------------------------
// Behavior 9: Until fires and immediately reverses when first walk
//             matches gate trigger
// ----------------------------------------------------------------

#[test]
fn until_fires_and_immediately_reverses_when_first_walk_matches_gate() {
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

    // Effect should have been fired then immediately reversed
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    let is_empty = stack.is_none() || stack.unwrap().is_empty();
    assert!(
        is_empty,
        "EffectStack should be empty — Until fires then immediately reverses when first walk matches gate"
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects after immediate fire-and-reverse"
    );
}

// ----------------------------------------------------------------
// Behavior 10: Until removal does not affect other entries in
//              BoundEffects
// ----------------------------------------------------------------

#[test]
fn until_removal_does_not_affect_other_bound_effects_entries() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![
        (
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        ),
        (
            "chip_b".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                }))),
            ),
        ),
    ]);
    world.entity_mut(entity).insert(bound);

    // First walk: NodeStartOccurred — Until fires, When does not match
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

    // Second walk: Bumped — Until reverses, When fires
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

    // SpeedBoost should be reversed (empty)
    let speed_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        speed_empty,
        "SpeedBoost should be reversed after Until gate trigger match"
    );

    // DamageBoost should have 1 entry from When firing
    let dmg_stack = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("DamageBoost EffectStack should exist from When firing");
    assert_eq!(
        dmg_stack.len(),
        1,
        "When tree should have fired DamageBoost"
    );

    // BoundEffects should only contain chip_b
    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a (Until) should be removed from BoundEffects"
    );
    assert!(
        remaining.iter().any(|(name, _)| name == "chip_b"),
        "chip_b (When) should remain in BoundEffects"
    );
}

// ----------------------------------------------------------------
// Behavior 11: Multiple Until entries with different sources track
//              independently
// ----------------------------------------------------------------

#[test]
fn multiple_until_entries_track_independently() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![
        (
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        ),
        (
            "chip_b".to_string(),
            Tree::Until(
                Trigger::BoltLostOccurred,
                Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                    DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            ),
        ),
    ]);
    world.entity_mut(entity).insert(bound);

    // First walk: NodeStartOccurred — both fire
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
        .expect("SpeedBoost should exist after both fire");
    assert_eq!(speed_stack.len(), 1);
    let dmg_stack = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("DamageBoost should exist after both fire");
    assert_eq!(dmg_stack.len(), 1);

    // Second walk: Bumped — only chip_a reverses
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
    assert!(
        speed_empty,
        "chip_a SpeedBoost should be reversed after Bumped trigger"
    );

    let dmg_stack_after = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .expect("chip_b DamageBoost should still be active");
    assert_eq!(
        dmg_stack_after.len(),
        1,
        "chip_b DamageBoost should still have 1 entry (BoltLostOccurred is its gate)"
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects"
    );
    assert!(
        remaining.iter().any(|(name, _)| name == "chip_b"),
        "chip_b should still be in BoundEffects"
    );
}

// Edge case for behavior 11: second Until also reverses on its gate trigger
#[test]
fn multiple_until_entries_both_reverse_on_respective_gate_triggers() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let bound = BoundEffects(vec![
        (
            "chip_a".to_string(),
            Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        ),
        (
            "chip_b".to_string(),
            Tree::Until(
                Trigger::BoltLostOccurred,
                Box::new(ScopedTree::Fire(ReversibleEffectType::DamageBoost(
                    DamageBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            ),
        ),
    ]);
    world.entity_mut(entity).insert(bound);

    // Walk 1: both fire
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

    // Walk 2: chip_a reverses
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

    // Walk 3: chip_b reverses
    let trees_third = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue3 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue3, &world);
        walk_bound_effects(
            entity,
            &Trigger::BoltLostOccurred,
            &TriggerContext::None,
            &trees_third,
            &mut commands,
        );
    }
    queue3.apply(&mut world);

    let speed_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    let dmg_empty = world
        .get::<EffectStack<DamageBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(speed_empty, "SpeedBoost should be reversed");
    assert!(dmg_empty, "DamageBoost should be reversed");

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        remaining.is_empty(),
        "BoundEffects should be empty after both Until entries reverse"
    );
}
