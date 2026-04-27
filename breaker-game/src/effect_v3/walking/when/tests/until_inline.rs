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

// ----------------------------------------------------------------
// Behavior 5 (REPLACED — D1): When(NodeStartOccurred, Until(TimeExpires(d),
// Fire(X))) on first matching outer trigger evaluates the Until inline,
// fires its inner, arms EffectTimers, marks UntilApplied, and binds the
// Until under the same source name. The Until is BOUND, not staged.
// ----------------------------------------------------------------
#[test]
fn when_evaluates_inner_until_inline_arming_timer() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "surge".to_string(),
        Tree::When(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(1.5)),
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

    // EffectStack: 1 entry from "surge", multiplier 2.0
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Until inner should fire inline");
    assert_eq!(stack.len(), 1);
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("surge"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));

    // EffectTimers: 1 entry (1.5, 1.5, "surge")
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be armed");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (OrderedFloat(1.5), OrderedFloat(1.5), test_source("surge"),)
    );

    // UntilApplied: contains "surge"
    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed");
    assert!(until_applied.0.contains("surge"));

    // BoundEffects: TWO entries — original When + self-bound Until
    let bound = world.get::<BoundEffects>(entity).unwrap();
    let surge_entries: Vec<&Tree> = bound
        .0
        .iter()
        .filter(|(name, _)| name == "surge")
        .map(|(_, t)| t)
        .collect();
    assert_eq!(
        surge_entries.len(),
        2,
        "BoundEffects must contain TWO surge entries: When outer + self-bound Until"
    );
    assert!(
        surge_entries
            .iter()
            .any(|t| matches!(t, Tree::When(Trigger::NodeStartOccurred, _))),
        "outer When entry must remain in BoundEffects"
    );
    assert!(
        surge_entries.iter().any(
            |t| matches!(t, Tree::Until(Trigger::TimeExpires(d), _) if *d == OrderedFloat(1.5))
        ),
        "a self-bound Until entry must be present in BoundEffects"
    );

    // StagedEffects MUST NOT contain a "surge" entry
    let staged_clean = world
        .get::<StagedEffects>(entity)
        .is_none_or(|s| s.0.iter().all(|(name, _)| name != "surge"));
    assert!(
        staged_clean,
        "Until must be BOUND, not staged — StagedEffects must not contain surge"
    );
}

// ----------------------------------------------------------------
// D1 edge case: walk the same entity a second time with the same outer
// trigger — must NOT multiply.
// ----------------------------------------------------------------
#[test]
fn when_inner_until_inline_second_outer_walk_does_not_multiply() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "surge".to_string(),
        Tree::When(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(1.5)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
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
        .expect("EffectStack should exist after both walks");
    assert_eq!(stack.len(), 1, "second walk must not duplicate the inner");

    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be present");
    assert_eq!(
        timers.timers.len(),
        1,
        "second walk must not duplicate timer"
    );

    let bound = world.get::<BoundEffects>(entity).unwrap();
    let when_count = bound
        .0
        .iter()
        .filter(|(n, t)| n == "surge" && matches!(t, Tree::When(..)))
        .count();
    let until_count = bound
        .0
        .iter()
        .filter(|(n, t)| n == "surge" && matches!(t, Tree::Until(..)))
        .count();
    assert_eq!(when_count, 1, "exactly ONE When outer entry");
    assert_eq!(until_count, 1, "exactly ONE Until self-bound entry");
}

// ----------------------------------------------------------------
// D2: three repeated outer triggers + one TimeExpires — Until reverses
// but outer When survives.
// ----------------------------------------------------------------
#[test]
fn when_inner_until_inline_three_outer_then_time_expires_reverses_only_inner() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "surge".to_string(),
        Tree::When(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(1.5)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
                    },
                ))),
            )),
        ),
    )]));

    for _ in 0..3 {
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

    // Sanity: still only one of each
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be present after three walks");
    assert_eq!(timers.timers.len(), 1);
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist");
    assert_eq!(stack.len(), 1);

    // Fourth walk: TimeExpires(1.5)
    let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        walk_bound_effects(
            entity,
            &Trigger::TimeExpires(OrderedFloat(1.5)),
            &TriggerContext::None,
            &trees,
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(stack_empty, "EffectStack should be empty after TimeExpires");

    let timers_clean = world.get::<EffectTimers>(entity).is_none_or(|t| {
        !t.timers
            .iter()
            .any(|(_, _, src)| *src == test_source("surge"))
    });
    assert!(
        timers_clean,
        "EffectTimers entry under surge should be cleared"
    );

    let until_applied = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("surge"));
    assert!(!until_applied, "UntilApplied should not contain surge");

    // Outer When MUST survive
    let bound = world.get::<BoundEffects>(entity).unwrap();
    let when_count = bound
        .0
        .iter()
        .filter(|(n, t)| n == "surge" && matches!(t, Tree::When(..)))
        .count();
    let until_count = bound
        .0
        .iter()
        .filter(|(n, t)| n == "surge" && matches!(t, Tree::Until(..)))
        .count();
    assert_eq!(
        when_count, 1,
        "outer When entry MUST survive TimeExpires reversal — required for re-arming"
    );
    assert_eq!(
        until_count, 0,
        "self-bound Until MUST be cleared on TimeExpires reversal"
    );
}

// ----------------------------------------------------------------
// D3: non-matching outer trigger does nothing.
// ----------------------------------------------------------------
#[test]
fn when_inner_until_inline_non_matching_outer_does_nothing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "surge".to_string(),
        Tree::When(
            Trigger::NodeStartOccurred,
            Box::new(Tree::Until(
                Trigger::TimeExpires(OrderedFloat(1.5)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(2.0),
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
                &Trigger::BoltLostOccurred,
                &TriggerContext::None,
                &trees,
                &mut commands,
            );
        }
        queue.apply(&mut world);
    }

    assert!(
        world.get::<EffectTimers>(entity).is_none(),
        "no EffectTimers should be armed without matching outer trigger"
    );
    assert!(
        world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
        "no inner should fire without matching outer trigger"
    );
    let until_chip = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("surge"));
    assert!(!until_chip, "UntilApplied should not contain surge");

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should contain only the original When entry"
    );
}

// ----------------------------------------------------------------
// D5: When(NodeStartOccurred, Until(Bumped, Fire(...))) — non-TimeExpires
// Until still routes inline (no timer armed, but Until evaluates and
// self-binds).
// ----------------------------------------------------------------
#[test]
fn when_evaluates_inner_until_inline_for_non_time_expires_gate() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_x".to_string(),
        Tree::When(
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
        .expect("Until inner should fire inline even for non-TimeExpires gate");
    assert_eq!(stack.len(), 1);
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("chip_x"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));

    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed");
    assert!(until_applied.0.contains("chip_x"));

    // BoundEffects: When outer + self-bound Until under chip_x
    let bound = world.get::<BoundEffects>(entity).unwrap();
    let chip_x_when = bound
        .0
        .iter()
        .filter(|(n, t)| n == "chip_x" && matches!(t, Tree::When(..)))
        .count();
    let chip_x_until = bound
        .0
        .iter()
        .filter(|(n, t)| n == "chip_x" && matches!(t, Tree::Until(Trigger::Bumped, _)))
        .count();
    assert_eq!(chip_x_when, 1, "outer When entry must remain");
    assert_eq!(chip_x_until, 1, "self-bound Until entry must be present");

    // EffectTimers MUST NOT be armed — gate is Bumped, not TimeExpires
    assert!(
        world.get::<EffectTimers>(entity).is_none(),
        "EffectTimers must NOT be armed for non-TimeExpires gate"
    );

    // StagedEffects MUST NOT contain chip_x
    let staged_clean = world
        .get::<StagedEffects>(entity)
        .is_none_or(|s| s.0.iter().all(|(name, _)| name != "chip_x"));
    assert!(staged_clean, "Until must be bound, not staged");
}

#[test]
fn when_inner_until_inline_non_time_expires_reverses_on_gate_match() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_x".to_string(),
        Tree::When(
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

    // Walk 2: Bumped — reverse
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::Bumped,
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(stack_empty, "EffectStack should be empty after Bumped");

    let until_applied = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("chip_x"));
    assert!(!until_applied, "UntilApplied should not contain chip_x");

    let bound = world.get::<BoundEffects>(entity).unwrap();
    let chip_x_when = bound
        .0
        .iter()
        .filter(|(n, t)| n == "chip_x" && matches!(t, Tree::When(..)))
        .count();
    let chip_x_until = bound
        .0
        .iter()
        .filter(|(n, t)| n == "chip_x" && matches!(t, Tree::Until(..)))
        .count();
    assert_eq!(
        chip_x_when, 1,
        "outer When MUST survive Bumped reversal of the inner Until"
    );
    assert_eq!(
        chip_x_until, 0,
        "self-bound Until MUST be cleared on Bumped reversal"
    );
}
