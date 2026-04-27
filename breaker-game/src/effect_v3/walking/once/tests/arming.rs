use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::effect_v3::{
    effects::SpeedBoostConfig,
    stacking::EffectStack,
    storage::{BoundEffects, StagedEffects},
    types::{EffectType, Tree, Trigger, TriggerContext},
    walking::walk_effects::walk_bound_effects,
};

// ----- Behavior 27 (Wave C): Once with gated inner arms the inner into
//       StagedEffects AND removes its own outer entry from BoundEffects.
//       Renamed from `once_removes_on_trigger_match_even_when_inner_tree_produces_no_effect`
//       to reflect the new arming semantic.

#[test]
fn once_arms_inner_when_gate_into_staged_and_removes_outer_from_bound() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Inner tree is a When(Died, ...) which is a trigger gate — under the
    // arming rules, Once(Bumped, When(Died, ...)) on an active Bumped
    // trigger should stage the inner When(Died, ...) and remove the outer
    // Once from BoundEffects.
    let bound = BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Once(
            Trigger::Bumped,
            Box::new(Tree::When(
                Trigger::Died,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            )),
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

    // No effect should fire — the inner is staged, not evaluated
    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "No effect should fire — inner When(Died, ...) is staged, not evaluated"
    );

    // Outer Once entry should be removed from BoundEffects
    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects — outer Once is one-shot"
    );

    // The inner When(Died, Fire(SpeedBoost)) should now be in StagedEffects
    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when Once arms its inner gate");
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::Died,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        ),
        "staged entry must be exactly the inner When(Died, Fire(SpeedBoost))"
    );
}

// ----------------------------------------------------------------
// Wave C behavior 10: Once(Bumped, When(Bumped, Fire(X))) — outer
// Once arms the inner When, then removes itself from BoundEffects.
// ----------------------------------------------------------------
#[test]
fn once_arms_inner_when_and_removes_outer_from_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Seed BoundEffects with the outer Once so the queued RemoveEffectCommand
    // has something to remove when applied.
    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        Tree::Once(
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
        evaluate_once(
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
        .expect("StagedEffects should be inserted when Once arms its inner gate");
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0],
        (
            "chip_a".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        ),
        "staged entry must be exactly the inner When subtree"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.iter().all(|(name, _)| name != "chip_a"),
        "outer Once entry must be removed from BoundEffects"
    );

    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "inner must NOT fire on the arming tick"
    );
}
