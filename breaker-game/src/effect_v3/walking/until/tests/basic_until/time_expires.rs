//! Group A — Top-level bound `Until(TimeExpires(d), Fire(...))` behaviors
//! (A1, A2, A3, A5). Bare-`World` + `CommandQueue` pattern, mirroring
//! `fire_and_reverse.rs` and `multi_entry.rs`.

use bevy::{ecs::world::CommandQueue, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        triggers::time::components::EffectTimers,
        types::{ReversibleEffectType, ScopedTree, Tree, Trigger, TriggerContext},
        walking::{UntilApplied, walk_effects::walk_bound_effects},
    },
    prelude::SourceId,
};

fn test_source(name: &str) -> SourceId {
    SourceId::from(name.to_owned())
}

fn until_time_expires_speed_tree(duration: f32, multiplier: f32) -> Tree {
    Tree::Until(
        Trigger::TimeExpires(OrderedFloat(duration)),
        Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
            SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            },
        ))),
    )
}

// ----------------------------------------------------------------
// A1: First walk on bound `Until(TimeExpires(2.0), Fire(SpeedBoost(1.5)))`
//     arms an `EffectTimers` entry, fires the inner, marks `UntilApplied`.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_first_walk_arms_timer_fires_inner_marks_applied() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
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

    // EffectStack: 1 entry with source "chip_a", multiplier 1.5
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after Until fires its inner");
    assert_eq!(stack.len(), 1, "Until should fire its inner exactly once");
    let entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(entries[0].0, test_source("chip_a"));
    assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));

    // EffectTimers: 1 entry (2.0, 2.0, "chip_a")
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be armed after first walk");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (OrderedFloat(2.0), OrderedFloat(2.0), test_source("chip_a"),)
    );

    // UntilApplied contains "chip_a"
    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed after first walk");
    assert!(until_applied.0.contains("chip_a"));

    // BoundEffects still contains the original Until entry under "chip_a"
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.iter().any(
            |(name, tree)| name == "chip_a" && *tree == until_time_expires_speed_tree(2.0, 1.5)
        ),
        "BoundEffects must retain the original Until entry under chip_a — only gate match removes it"
    );
}

#[test]
fn until_time_expires_idempotent_arming_on_second_non_matching_walk() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
    )]));

    // Walk 1: NodeStartOccurred (non-matching gate) — arms
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

    // Walk 2: BoltLostOccurred (also non-matching) — must NOT duplicate
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::BoltLostOccurred,
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should still be present");
    assert_eq!(
        timers.timers.len(),
        1,
        "second non-matching walk must NOT add a duplicate timer entry"
    );
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should still be present");
    assert_eq!(
        stack.len(),
        1,
        "second non-matching walk must NOT fire the inner again"
    );
}

// ----------------------------------------------------------------
// A2: Reversal on `TimeExpires(d)` clears stack, removes `EffectTimers`
//     entry, removes `UntilApplied` source, removes `BoundEffects` entry.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_gate_match_after_arm_reverses_and_cleans_up() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
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

    // Pre-condition: arming must have installed EffectTimers (precondition for A2)
    let timers_pre = world
        .get::<EffectTimers>(entity)
        .expect("walk 1 must arm EffectTimers — preconditional assertion");
    assert_eq!(
        timers_pre.timers.len(),
        1,
        "walk 1 must produce exactly one EffectTimers entry"
    );

    // Walk 2: TimeExpires(2.0) — reverse
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::TimeExpires(OrderedFloat(2.0)),
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    // EffectStack: empty or absent
    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(stack_empty, "EffectStack should be empty after reversal");

    // EffectTimers: absent or no entry under chip_a
    let timers_clean = match world.get::<EffectTimers>(entity) {
        None => true,
        Some(timers) => !timers
            .timers
            .iter()
            .any(|(_, _, src)| *src == test_source("chip_a")),
    };
    assert!(
        timers_clean,
        "EffectTimers entry under chip_a should be cleared on reversal"
    );

    // UntilApplied: chip_a removed
    let until_applied_chip_a = world
        .get::<UntilApplied>(entity)
        .is_some_and(|ua| ua.0.contains("chip_a"));
    assert!(
        !until_applied_chip_a,
        "UntilApplied should not contain chip_a after reversal"
    );

    // BoundEffects: no entry named chip_a
    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert!(
        bound.0.iter().all(|(name, _)| name != "chip_a"),
        "BoundEffects entry chip_a should be removed on reversal"
    );
}

#[test]
fn until_time_expires_third_walk_after_reversal_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
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

    // Walk 2: TimeExpires(2.0) — reverse
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::TimeExpires(OrderedFloat(2.0)),
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    // Walk 3: TimeExpires(2.0) again — must be a no-op
    let trees3 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue3 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue3, &world);
        walk_bound_effects(
            entity,
            &Trigger::TimeExpires(OrderedFloat(2.0)),
            &TriggerContext::None,
            &trees3,
            &mut commands,
        );
    }
    queue3.apply(&mut world);

    let stack_empty = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .is_none_or(EffectStack::is_empty);
    assert!(
        stack_empty,
        "third walk after reversal must not reinstate stack"
    );

    let timers = world.get::<EffectTimers>(entity);
    let no_chip_a = timers.is_none_or(|t| {
        !t.timers
            .iter()
            .any(|(_, _, src)| *src == test_source("chip_a"))
    });
    assert!(
        no_chip_a,
        "third walk after reversal must not reinstate EffectTimers entry"
    );
}

// ----------------------------------------------------------------
// A3: Five subsequent non-matching walks do NOT multiply timer entries
//     or stack entries.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_five_non_matching_walks_do_not_multiply() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
    )]));

    let triggers = [
        Trigger::NodeStartOccurred,
        Trigger::BoltLostOccurred,
        Trigger::NodeEndOccurred,
        Trigger::BumpOccurred,
        Trigger::PerfectBumpOccurred,
    ];

    for trig in &triggers {
        let trees = world.get::<BoundEffects>(entity).unwrap().0.clone();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_bound_effects(entity, trig, &TriggerContext::None, &trees, &mut commands);
        }
        queue.apply(&mut world);
    }

    // EffectStack: exactly 1
    let stack = world
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("EffectStack should exist after 5 walks");
    assert_eq!(
        stack.len(),
        1,
        "five non-matching walks must yield exactly 1 stack entry"
    );

    // EffectTimers: exactly 1 entry
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should be present after 5 walks");
    assert_eq!(timers.timers.len(), 1);
    assert_eq!(
        timers.timers[0],
        (OrderedFloat(2.0), OrderedFloat(2.0), test_source("chip_a"),)
    );

    // UntilApplied: exactly the source "chip_a"
    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be installed");
    assert!(until_applied.0.contains("chip_a"));
    assert_eq!(
        until_applied.0.len(),
        1,
        "UntilApplied should contain exactly chip_a"
    );
}

// ----------------------------------------------------------------
// A5: BoundEffects retains the ("chip_a", Tree::Until(...)) entry while
//     the timer is ticking.
// ----------------------------------------------------------------

#[test]
fn until_time_expires_bound_effects_retains_entry_while_armed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.entity_mut(entity).insert(BoundEffects(vec![(
        "chip_a".to_string(),
        until_time_expires_speed_tree(2.0, 1.5),
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

    // Walk 2: BumpOccurred (non-matching gate) — must NOT remove from BoundEffects
    let trees2 = world.get::<BoundEffects>(entity).unwrap().0.clone();
    let mut queue2 = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue2, &world);
        walk_bound_effects(
            entity,
            &Trigger::BumpOccurred,
            &TriggerContext::None,
            &trees2,
            &mut commands,
        );
    }
    queue2.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects must contain exactly one entry — the original chip_a Until"
    );
    assert_eq!(
        bound.0[0],
        (
            "chip_a".to_string(),
            until_time_expires_speed_tree(2.0, 1.5),
        ),
        "the entry must be structurally equal to the original Until tree"
    );

    // EffectTimers: still exactly 1 entry — the second walk did NOT enqueue a duplicate
    let timers = world
        .get::<EffectTimers>(entity)
        .expect("EffectTimers should still be present");
    assert_eq!(
        timers.timers.len(),
        1,
        "second non-matching walk must NOT enqueue a duplicate timer"
    );

    // UntilApplied is the gate guarding re-arming
    let until_applied = world
        .get::<UntilApplied>(entity)
        .expect("UntilApplied should be present");
    assert!(until_applied.0.contains("chip_a"));
}
