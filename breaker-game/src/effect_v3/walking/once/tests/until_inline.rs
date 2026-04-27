use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        triggers::time::components::EffectTimers,
        types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::SourceId,
};

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
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
