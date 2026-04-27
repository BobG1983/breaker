use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use super::system::*;
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        triggers::time::components::EffectTimers,
        types::{
            EffectType, ReversibleEffectType, ScopedTree, Terminal, Tree, Trigger, TriggerContext,
        },
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::{DamageBoostStack, SourceId},
};

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

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
        (dmg_stack.aggregate_persistent() - 2.0).abs() < 1e-5,
        "DamageBoostStack aggregate should be 2.0, got {}",
        dmg_stack.aggregate_persistent()
    );

    let remaining = &world.get::<BoundEffects>(entity).unwrap().0;
    assert!(
        !remaining.iter().any(|(name, _)| name == "chip_a"),
        "chip_a should be removed from BoundEffects after Once fired"
    );
}

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

// ================================================================
// Group E — Once(_, Until(...)) routes inline + removes outer
// ================================================================

// ----------------------------------------------------------------
// E1: Once(NodeStartOccurred, Until(TimeExpires(3.0), Fire(SpeedBoost(2.0))))
//     — first matching outer trigger removes the Once outer, evaluates the
//     Until inline, arms EffectTimers, marks UntilApplied, binds the Until
//     under the same source name.
// ----------------------------------------------------------------
#[test]
fn once_evaluates_inner_until_inline_and_removes_outer() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "kickstart".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(3.0)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            )),
        ),
    )]));

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

    // EffectStack: 1 entry from "kickstart", multiplier 2.0
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Until inner should fire inline");
    assert_eq!(stack.len(), 1);
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("kickstart"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));

    // EffectTimers: 1 entry (3.0, 3.0, "kickstart")
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be armed");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (
            OrderedFloat(3.0),
            OrderedFloat(3.0),
            test_source("kickstart"),
        )
    );

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed");
    assert!(until_applied.0.contains("kickstart"));

    // BoundEffects: exactly ONE entry under "kickstart" — the Until self-binding
    // (the original Once was removed)
    let bound = world.get::<BoundEffects>(entity).unwrap();
    let kickstart_entries: Vec<&Tree> = bound
        .0
        .iter()
        .filter(|(name, _)| name == "kickstart")
        .map(|(_, t)| t)
        .collect();
    assert_eq!(
        kickstart_entries.len(),
        1,
        "BoundEffects should contain exactly ONE kickstart entry — the Until self-binding"
    );
    assert!(
        matches!(
            kickstart_entries[0],
            Tree::Until(Trigger::TimeExpires(d), _) if *d == OrderedFloat(3.0)
        ),
        "the remaining kickstart entry must be the Until self-binding, NOT the original Once"
    );
    assert!(
        bound.0.iter().all(|(_, t)| !matches!(t, Tree::Once(..))),
        "the original Once outer must be removed"
    );

    // StagedEffects: kickstart not in there
    let staged_clean = world
        .get::<StagedEffects>(entity)
        .is_none_or(|s| s.0.iter().all(|(name, _)| name != "kickstart"));
    assert!(
        staged_clean,
        "Until must be BOUND, not staged, after Once routes it inline"
    );
}

// ----------------------------------------------------------------
// E2: After E1's end state, a second NodeStartOccurred is a no-op for the
// (already-removed) Once outer; the bound Until is unaffected.
// ----------------------------------------------------------------
#[test]
fn once_inner_until_inline_second_node_start_does_nothing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "kickstart".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(3.0)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            )),
        ),
    )]));

    // Walk 1: arm + remove outer
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

    // Walk 2: NodeStartOccurred again — should be a no-op for the gone Once
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees2,
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
        "second walk must NOT add another stack entry"
    );

    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should still be present");
    assert_eq!(
        timers.timers.len(),
        1,
        "second walk must NOT duplicate timer"
    );

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should still be present");
    assert!(until_applied.0.contains("kickstart"));
    assert_eq!(
        until_applied.0.len(),
        1,
        "UntilApplied should contain exactly kickstart"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    let kickstart_entries: Vec<&Tree> = bound
        .0
        .iter()
        .filter(|(name, _)| name == "kickstart")
        .map(|(_, t)| t)
        .collect();
    assert_eq!(kickstart_entries.len(), 1);
    assert!(matches!(kickstart_entries[0], Tree::Until(..)));
}

// ----------------------------------------------------------------
// E3: TimeExpires(3.0) after E1 reverses the Until and clears all state.
// ----------------------------------------------------------------
#[test]
fn once_inner_until_inline_time_expires_clears_all_state() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "kickstart".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(3.0)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            )),
        ),
    )]));

    // Walk 1: arm
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

    // Walk 2: TimeExpires(3.0)
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::TimeExpires(OrderedFloat(3.0)),
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    assert_kickstart_fully_gone(&world, entity);

    // Walk 3: NodeStartOccurred AGAIN — the Once is gone forever, must be a
    // complete no-op. No state should be restored, nothing re-armed.
    let trees3 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue3 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue3, &world);
        walk_bound_effects(
            entity,
            &Trigger::NodeStartOccurred,
            &TriggerContext::None,
            &trees3,
            &mut commands,
        );
    }
    queue3.apply(&mut world);

    assert_kickstart_fully_gone(&world, entity);
}

/// Assert all four "kickstart fully gone" invariants on `entity`:
/// `SpeedBoost` stack empty, no `EffectTimers` entry under the kickstart
/// source, `UntilApplied` does not contain "kickstart", and `BoundEffects`
/// has no entry named "kickstart".
fn assert_kickstart_fully_gone(world: &World, entity: Entity) {
    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(stack_empty, "SpeedBoost stack must be empty");

    let timers_clean = world.get::<EffectTimers>(entity).is_none_or(|t| {
        !t.timers
            .iter()
            .any(|(_, _, src)| *src == test_source("kickstart"))
    });
    assert!(timers_clean, "EffectTimers must not contain kickstart");

    let until_applied = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("kickstart"));
    assert!(!until_applied, "UntilApplied must not contain kickstart");

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.iter().all(|(name, _)| name != "kickstart"),
        "BoundEffects must not contain kickstart"
    );
}

// ----------------------------------------------------------------
// E5: Non-matching outer trigger does nothing.
// ----------------------------------------------------------------
#[test]
fn once_inner_until_inline_non_matching_outer_does_nothing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "kickstart".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(3.0)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            )),
        ),
    )]));

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

    assert!(world.get::<EffectTimers>(entity).is_none());
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    let until_chip = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("kickstart"));
    assert!(!until_chip);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert!(matches!(bound.0[0].1, Tree::Once(_, _)));
}

// ----------------------------------------------------------------
// E6: Once(NodeStartOccurred, Until(Bumped, Fire(...))) — non-TimeExpires
// Until still routes inline + outer Once removed.
// ----------------------------------------------------------------
#[test]
fn once_evaluates_inner_until_inline_for_non_time_expires_gate() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_x".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            )),
        ),
    )]));

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
        .expect("Until inner should fire inline");
    assert_eq!(stack.len(), 1);
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("chip_x"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed");
    assert!(until_applied.0.contains("chip_x"));

    // BoundEffects: ONE entry under chip_x — the Until self-binding (Once removed)
    let bound = world.get::<BoundEffects>(entity).unwrap();
    let chip_x_entries: Vec<&Tree> = bound
        .0
        .iter()
        .filter(|(name, _)| name == "chip_x")
        .map(|(_, t)| t)
        .collect();
    assert_eq!(chip_x_entries.len(), 1);
    assert!(matches!(chip_x_entries[0], Tree::Until(Trigger::Bumped, _)));

    // No EffectTimers, no StagedEffects under chip_x
    assert!(
        world.get::<EffectTimers>(entity).is_none(),
        "EffectTimers must NOT be armed for non-TimeExpires gate"
    );
    let staged_clean = world
        .get::<StagedEffects>(entity)
        .is_none_or(|s| s.0.iter().all(|(name, _)| name != "chip_x"));
    assert!(staged_clean);
}

#[test]
fn once_inner_until_inline_non_time_expires_second_outer_walk_no_op() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_x".to_string(),
        Tree::Once(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::Bumped,
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            )),
        ),
    )]));

    for _ in 0..2 {
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
    }

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist");
    assert_eq!(
        stack.len(),
        1,
        "second NodeStartOccurred must be a no-op — the Once outer was already removed"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    let chip_x_until = bound
        .0
        .iter()
        .filter(|(name, t)| name == "chip_x" && matches!(t, Tree::Until(..)))
        .count();
    let chip_x_once = bound
        .0
        .iter()
        .filter(|(name, t)| name == "chip_x" && matches!(t, Tree::Once(..)))
        .count();
    assert_eq!(chip_x_until, 1, "Until self-binding survives second walk");
    assert_eq!(chip_x_once, 0, "Once outer was already removed");
}
