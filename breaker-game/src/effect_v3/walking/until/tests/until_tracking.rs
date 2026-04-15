use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::effect_v3::{
    effects::SpeedBoostConfig,
    storage::BoundEffects,
    types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
    walking::walk_effects::walk_bound_effects,
};

// ----------------------------------------------------------------
// Behavior 12: UntilApplied component is created on first Until
//              evaluation if not present
// ----------------------------------------------------------------

#[test]
fn until_applied_component_created_on_first_evaluation() {
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

    assert!(
        world.get::<UntilApplied>(entity).is_none(),
        "UntilApplied should not exist before first walk"
    );

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

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied component should be inserted after first evaluation");
    assert!(
        until_applied.0.contains("chip_a"),
        "UntilApplied should contain 'chip_a' after first evaluation"
    );
}

// Edge case for behavior 12: entity already has UntilApplied from different source
#[test]
fn until_applied_adds_to_existing_set_from_different_source() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Pre-insert UntilApplied with a different source
    let mut existing = UntilApplied::default();
    existing.0.insert("chip_x".to_string());
    world.entity_mut(entity).insert(existing);

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

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should exist");
    assert!(
        until_applied.0.contains("chip_a"),
        "chip_a should be added to existing UntilApplied"
    );
    assert!(
        until_applied.0.contains("chip_x"),
        "chip_x should remain in UntilApplied"
    );
}

// ----------------------------------------------------------------
// Behavior 13: UntilApplied source entry is removed after reversal
// ----------------------------------------------------------------

#[test]
fn until_applied_source_removed_after_reversal() {
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

    // Verify it's in UntilApplied
    assert!(
        world
            .get::<UntilApplied>(entity)
            .expect("UntilApplied should exist")
            .0
            .contains("chip_a"),
        "chip_a should be in UntilApplied after first walk"
    );

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

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied component should still exist (may be empty)");
    assert!(
        !until_applied.0.contains("chip_a"),
        "chip_a should be removed from UntilApplied after reversal"
    );
}
