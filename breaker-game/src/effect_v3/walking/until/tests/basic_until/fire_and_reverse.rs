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
// Behavior 1: Until fires inner ScopedTree::Fire effect on first
//             evaluation (any trigger)
// ----------------------------------------------------------------

#[test]
fn until_fires_inner_fire_effect_on_first_walk_with_non_matching_trigger() {
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
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after Until fires on first walk");
    assert_eq!(
        stack.len(),
        1,
        "Until should fire inner effect on first walk regardless of trigger"
    );
}

// ----------------------------------------------------------------
// Behavior 2: Until does not fire inner effect a second time on
//             subsequent non-matching walks
// ----------------------------------------------------------------

#[test]
fn until_does_not_fire_inner_effect_on_subsequent_non_matching_walk() {
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

    // First walk: fires inner effect
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

    // Second walk: different non-matching trigger
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
        .expect("EffectStack should still exist");
    assert_eq!(
        stack.len(),
        1,
        "Until should NOT fire inner effect a second time on subsequent non-matching walk"
    );
}

// Edge case for behavior 2: 5 non-matching walks still yields exactly 1 stack entry
#[test]
fn until_does_not_fire_on_five_subsequent_non_matching_walks() {
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

    let non_matching_triggers = [
        Trigger::NodeStartOccurred,
        Trigger::BoltLostOccurred,
        Trigger::NodeEndOccurred,
        Trigger::BumpOccurred,
        Trigger::PerfectBumpOccurred,
    ];

    for trigger in &non_matching_triggers {
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_bound_effects(
                entity,
                trigger,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);
    }

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after 5 walks");
    assert_eq!(
        stack.len(),
        1,
        "After 5 non-matching walks, stack should still have exactly 1 entry"
    );
}

// ----------------------------------------------------------------
// Behavior 3: Until reverses inner effect when gate trigger fires
//             (after prior application)
// ----------------------------------------------------------------

#[test]
fn until_reverses_inner_effect_when_gate_trigger_matches_after_application() {
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

    // Second walk: gate trigger matches — should reverse
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

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    let is_empty = stack.is_none() || stack.unwrap().is_empty();
    assert!(
        is_empty,
        "EffectStack should be empty after Until reverses on gate trigger match"
    );
}
