use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::system::*;
use crate::effect_v3::{
    effects::SpeedBoostConfig,
    stacking::EffectStack,
    storage::{BoundEffects, StagedEffects},
    types::{EffectType, Tree, Trigger, TriggerContext},
};

#[test]
fn walk_effects_fire_tree_queues_fire_command() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let trees = vec![(
        "test_chip".to_string(),
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    )];

    let trigger = Trigger::Bumped;
    let context = TriggerContext::None;

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
    }
    queue.apply(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after walk_bound_effects fires a SpeedBoost");
    assert_eq!(stack.len(), 1);
}

#[test]
fn walk_effects_when_matching_trigger_fires_inner() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let trees = vec![(
        "test_chip".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
    )];

    let trigger = Trigger::Bumped;
    let context = TriggerContext::None;

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
    }
    queue.apply(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after matching When trigger");
    assert_eq!(stack.len(), 1);
}

#[test]
fn walk_effects_when_non_matching_trigger_does_nothing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let trees = vec![(
        "test_chip".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
    )];

    // Use a different trigger that doesn't match
    let trigger = Trigger::BoltLostOccurred;
    let context = TriggerContext::None;

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "No EffectStack should exist when trigger doesn't match"
    );
}

// ================================================================
// Wave C — walk_staged_effects
// ================================================================

// ----------------------------------------------------------------
// Behavior 15: walk_staged_effects with a matching trigger consumes
//              the entry via remove_effect
// ----------------------------------------------------------------
#[test]
fn walk_staged_effects_matching_trigger_consumes_entry() {
    let mut world = World::new();
    let entity = world
        .spawn((
            StagedEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )]),
            BoundEffects(vec![]),
        ))
        .id();

    let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_staged_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &staged_snapshot,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("inner Fire should have run");
    assert_eq!(stack.len(), 1);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert!(
        staged.0.iter().all(|(name, _)| name != "chip_a"),
        "staged entry must be consumed via RemoveEffectCommand"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty — only the staged copy was removed"
    );
}

// ----------------------------------------------------------------
// Behavior 16: walk_staged_effects with a non-matching trigger
//              leaves the entry alone
// ----------------------------------------------------------------
#[test]
fn walk_staged_effects_non_matching_trigger_leaves_entry_alone() {
    let mut world = World::new();
    let entity = world
        .spawn((
            StagedEffects(vec![(
                "chip_a".to_string(),
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )]),
            BoundEffects(vec![]),
        ))
        .id();

    let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_staged_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &staged_snapshot,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "inner must not fire when inner gate trigger does not match"
    );

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::BoltLostOccurred,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )
    );
}

// ----------------------------------------------------------------
// Behavior 17: walk_staged_effects defensively handles a staged
//              Tree::Fire (non-gate root) as a no-op
// ----------------------------------------------------------------
#[test]
fn walk_staged_effects_defensively_ignores_non_gate_root() {
    let mut world = World::new();
    let entity = world
        .spawn((
            StagedEffects(vec![(
                "chip_a".to_string(),
                Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                })),
            )]),
            BoundEffects(vec![]),
        ))
        .id();

    let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_staged_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &staged_snapshot,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "non-gate staged Fire must not fire"
    );

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0].0, "chip_a");
}
