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
            BumpTarget, Condition, EffectType, ParticipantTarget, ReversibleEffectType, ScopedTree,
            Terminal, Tree, Trigger, TriggerContext,
        },
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::{DamageBoostStack, SourceId},
};

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

#[test]
fn evaluate_when_matching_trigger_fires_inner() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let gate = Trigger::Bumped;
    let active = Trigger::Bumped;
    let context = TriggerContext::None;

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &gate,
            &inner,
            &active,
            &context,
            "test_chip",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity).unwrap();
    assert_eq!(stack.len(), 1);
}

#[test]
fn evaluate_when_non_matching_trigger_does_nothing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));
    let gate = Trigger::Bumped;
    let active = Trigger::BoltLostOccurred;
    let context = TriggerContext::None;

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &gate,
            &inner,
            &active,
            &context,
            "test_chip",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(stack.is_none());
}

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

// ----------------------------------------------------------------
// Behavior 6: When(Bumped, Fire(X)) — non-gate Fire inner still
//             evaluates recursively (regression)
// ----------------------------------------------------------------
#[test]
fn when_non_gate_fire_inner_fires_immediately_not_armed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(1.5),
    }));

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

    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("Fire inner should fire immediately");
    assert_eq!(stack.len(), 1);
    assert!(
        world.get::<StagedEffects>(entity).is_none(),
        "non-gate Fire inner must NOT be staged"
    );
}

// ----------------------------------------------------------------
// Behavior 7: When(Bumped, Sequence([Fire(X), Fire(Y)])) — non-gate
//             Sequence inner still evaluates recursively
// ----------------------------------------------------------------
#[test]
fn when_non_gate_sequence_inner_fires_immediately_not_armed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::Sequence(vec![
        Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        })),
    ]);

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

    let speed = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("SpeedBoost should have fired");
    assert_eq!(speed.len(), 1);
    let dmg = world
        .get::<DamageBoostStack>(entity)
        .expect("DamageBoost should have fired");
    assert!(!dmg.is_empty());
    assert!(
        (dmg.aggregate_persistent() - 2.0).abs() < 1e-5,
        "DamageBoostStack aggregate should be 2.0, got {}",
        dmg.aggregate_persistent()
    );
    assert!(
        world.get::<StagedEffects>(entity).is_none(),
        "Sequence inner must not be staged"
    );
}

// ----------------------------------------------------------------
// Behavior 8: When(Bumped, On(Bump(Bolt), Fire(X))) — non-gate On
//             inner still evaluates recursively
// ----------------------------------------------------------------
#[test]
fn when_non_gate_on_inner_redirects_immediately_not_armed() {
    let mut world = World::new();
    let breaker = world.spawn_empty().id();
    let bolt = world.spawn_empty().id();

    let inner = Tree::On(
        ParticipantTarget::Bump(BumpTarget::Bolt),
        Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            breaker,
            &Trigger::Bumped,
            &inner,
            &Trigger::Bumped,
            &TriggerContext::Bump {
                bolt: Some(bolt),
                breaker,
            },
            "chip_a",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let bolt_stack = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("On should have redirected effect to bolt");
    assert_eq!(bolt_stack.len(), 1);
    assert!(
        world
            .get::<EffectStack<SpeedBoostConfig>>(breaker)
            .is_none(),
        "breaker should not have received effect"
    );
    assert!(world.get::<StagedEffects>(bolt).is_none());
    assert!(world.get::<StagedEffects>(breaker).is_none());
}

// ----------------------------------------------------------------
// Behavior 9: When(Bumped, During(cond, ...)) — non-gate During
//             inner still recurses (arming regression guard)
// ----------------------------------------------------------------
#[test]
fn when_non_gate_during_inner_is_not_armed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::During(
        Condition::NodeActive,
        Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ))),
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

    assert!(
        world.get::<StagedEffects>(entity).is_none(),
        "During inner must NOT be armed — only trigger gates (When/Once/Until) are"
    );
}

// ----------------------------------------------------------------
// Behavior 12: Staging uses the same source name as the outer entry
// ----------------------------------------------------------------
#[test]
fn when_arming_uses_same_source_name_as_outer_entry() {
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
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_xyz",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0[0].0, "chip_xyz",
        "staged name must be the exact source passed to evaluate_when"
    );
}

// ----------------------------------------------------------------
// Behavior 13: Two direct evaluate_when calls stage independent
//              entries with distinct source names
// ----------------------------------------------------------------
#[test]
fn when_arming_two_calls_append_independent_entries_in_order() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner_a = Tree::When(
        Trigger::Bumped,
        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        }))),
    );
    let inner_b = Tree::When(
        Trigger::Bumped,
        Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        }))),
    );

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner_a,
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_a",
            &mut commands,
        );
        evaluate_when(
            entity,
            &Trigger::Bumped,
            &inner_b,
            &Trigger::Bumped,
            &TriggerContext::None,
            "chip_b",
            &mut commands,
        );
    }
    queue.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 2);
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
        )
    );
    assert_eq!(
        staged.0[1],
        (
            "chip_b".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                }))),
            ),
        )
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
    assert!(
        world.get::<DamageBoostStack>(entity).is_none(),
        "DamageBoostStack must not be inserted — inner When is staged, not fired"
    );
}

// ----------------------------------------------------------------
// Behavior 14: Triple-nested When — a single evaluate_when call
//              arms only one layer
// ----------------------------------------------------------------
#[test]
fn when_arming_single_call_arms_only_one_layer() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let inner = Tree::When(
        Trigger::Bumped,
        Box::new(Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        )),
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
                Trigger::Bumped,
                Box::new(Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                )),
            ),
        )
    );
    assert!(world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none());
}
