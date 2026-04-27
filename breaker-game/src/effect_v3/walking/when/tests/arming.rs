use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::effect_v3::{
    effects::SpeedBoostConfig,
    stacking::EffectStack,
    storage::{BoundEffects, StagedEffects},
    types::{EffectType, Tree, Trigger, TriggerContext},
};

// ================================================================
// Wave C — When-arming for nested trigger gates
// ================================================================

// ----------------------------------------------------------------
// Behavior 1: When(Bumped, When(Bumped, Fire(X))) — first matching
//             trigger arms the inner When
// ----------------------------------------------------------------
#[test]
fn when_arms_inner_when_on_first_matching_trigger() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            )),
        ),
    )]));

    let inner = Tree::When(
        Trigger::Bumped,
        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }))),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner,
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_a",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when inner gate is armed");
    assert_eq!(staged.0.len(), 1);
    assert_eq!(staged.0[0].0, "chip_a");
    assert_eq!(
        staged.0[0].1,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        "staged entry must be exactly the inner When subtree"
    );

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "inner must NOT fire on the arming tick"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(bound.0[0].0, "chip_a");
}

// ----------------------------------------------------------------
// Behavior 2: Non-matching outer trigger arms nothing
// ----------------------------------------------------------------
#[test]
fn when_arming_does_nothing_on_non_matching_outer_trigger() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::When(
        Trigger::Bumped,
        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }))),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner,
            &Trigger::BoltLostOccurred,
            &TriggerContext::None,
            "chip_a",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    assert!(
        world.get::<StagedEffects>(entity).is_none(),
        "no StagedEffects should be inserted when outer trigger does not match"
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
}

// ----------------------------------------------------------------
// Behavior 3: When(Bumped, When(NoBumpOccurred, Fire(X))) — different
//             inner trigger still arms
// ----------------------------------------------------------------
#[test]
fn when_arms_inner_when_with_different_inner_trigger() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::When(
        Trigger::NoBumpOccurred,
        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }))),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner,
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_a",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::NoBumpOccurred,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
}

// ----------------------------------------------------------------
// Behavior 4: When(Bumped, Once(Bumped, Fire(X))) — arms the inner
//             Once (preserves variant)
// ----------------------------------------------------------------
#[test]
fn when_arms_inner_once_preserving_variant() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::Once(
        Trigger::Bumped,
        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }))),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner,
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_a",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::Once(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
}
