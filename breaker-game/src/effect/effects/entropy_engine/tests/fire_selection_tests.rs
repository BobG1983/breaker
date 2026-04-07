//! Tests for `fire()` selection behavior: independent selection, determinism,
//! edge cases (`max_effects=0`, all-zero weights), mixed `Do`/non-`Do` dispatch,
//! and `StagedEffects` insertion when absent.

use bevy::prelude::*;

use crate::{
    effect::{
        EffectNode,
        core::{EffectKind, StagedEffects, Trigger},
        effects::{
            damage_boost::ActiveDamageBoosts, entropy_engine::effect::*,
            speed_boost::ActiveSpeedBoosts,
        },
    },
    shared::rng::GameRng,
};

// -- Behavior 14: fire() selects N effects independently -- same effect can repeat --

#[test]
fn fire_selects_effects_independently_total_equals_n() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 2 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();
    let pool = vec![
        (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            1.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ];

    fire(entity, 5, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 3,
        "cells_destroyed should be 3 (2 + 1)"
    );

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    let total = damage.0.len() + speed.0.len();
    assert_eq!(
        total, 3,
        "exactly 3 effects should fire (min(3, 5)=3), got {} (damage: {:?}, speed: {:?})",
        total, damage.0, speed.0
    );
}

// -- Behavior 15: fire() with non-Do pool entry pushes to StagedEffects --

#[test]
fn fire_with_non_do_pushes_to_staged_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 0 },
            StagedEffects::default(),
        ))
        .id();
    let pool = vec![(1.0, non_do_node.clone())];

    fire(entity, 3, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(state.cells_destroyed, 1, "cells_destroyed should be 1");

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "1 non-Do effect should be pushed to StagedEffects (min(1, 3)=1)"
    );
    assert_eq!(staged.0[0].0, "", "chip name should be empty string");
    assert_eq!(
        staged.0[0].1, non_do_node,
        "the When node should be pushed to StagedEffects"
    );
}

// -- Behavior 16: fire() is deterministic for the same seed --

#[test]
fn fire_deterministic_for_same_seed() {
    let pool = vec![
        (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            2.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ];

    // World 1
    let mut world1 = World::new();
    world1.insert_resource(GameRng::from_seed(77));
    let entity1 = world1
        .spawn((
            EntropyEngineState { cells_destroyed: 4 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();
    fire(entity1, 5, &pool, "", &mut world1);

    // World 2
    let mut world2 = World::new();
    world2.insert_resource(GameRng::from_seed(77));
    let entity2 = world2
        .spawn((
            EntropyEngineState { cells_destroyed: 4 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();
    fire(entity2, 5, &pool, "", &mut world2);

    let damage1 = world1.get::<ActiveDamageBoosts>(entity1).unwrap();
    let damage2 = world2.get::<ActiveDamageBoosts>(entity2).unwrap();
    let speed1 = world1.get::<ActiveSpeedBoosts>(entity1).unwrap();
    let speed2 = world2.get::<ActiveSpeedBoosts>(entity2).unwrap();
    let state1 = world1.get::<EntropyEngineState>(entity1).unwrap();
    let state2 = world2.get::<EntropyEngineState>(entity2).unwrap();

    assert_eq!(
        state1.cells_destroyed, 5,
        "world1 cells_destroyed should be 5"
    );
    assert_eq!(
        state2.cells_destroyed, 5,
        "world2 cells_destroyed should be 5"
    );
    assert_eq!(
        damage1.0, damage2.0,
        "ActiveDamageBoosts must match across worlds with same seed"
    );
    assert_eq!(
        speed1.0, speed2.0,
        "ActiveSpeedBoosts must match across worlds with same seed"
    );
}

// -- Behavior 17: fire() with max_effects=0 fires zero effects --

#[test]
fn fire_with_max_effects_zero_fires_nothing() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 3 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 0, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 4,
        "cells_destroyed should increment to 4 even with max_effects=0"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        active.0.is_empty(),
        "no effects should fire with max_effects=0"
    );
}

// -- Behavior 18: fire() with all-zero weights increments cells_destroyed but fires nothing --

#[test]
fn fire_with_all_zero_weights_increments_cells_destroyed_but_no_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 2 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();
    let pool = vec![
        (0.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            0.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ];

    fire(entity, 5, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 3,
        "cells_destroyed should increment to 3 even with all-zero weights"
    );
    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(damage.0.is_empty(), "no DamageBoost with all-zero weights");
    assert!(speed.0.is_empty(), "no SpeedBoost with all-zero weights");
}

// -- Behavior 23: fire() works with mixed Do and non-Do pool entries --

#[test]
fn fire_with_mixed_do_and_non_do_dispatches_correctly() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let non_do_node = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    };
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 1 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();
    let pool = vec![
        (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (1.0, non_do_node),
    ];

    fire(entity, 5, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 2,
        "cells_destroyed should be 2 (1 + 1)"
    );

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let staged = world.get::<StagedEffects>(entity).unwrap();
    let total = damage.0.len() + staged.0.len();
    assert_eq!(
        total, 2,
        "exactly 2 dispatches (min(2, 5)=2): damage entries + staged entries = {}, got (damage: {:?}, staged: {:?})",
        total, damage.0, staged.0
    );
}

// -- Behavior 24: fire() inserts StagedEffects fresh when absent and non-Do selected --

#[test]
fn fire_inserts_staged_effects_when_absent_and_non_do_selected() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    // Entity has EntropyEngineState but NO StagedEffects
    let entity = world.spawn(EntropyEngineState { cells_destroyed: 0 }).id();
    let pool = vec![(1.0, non_do_node.clone())];

    fire(entity, 3, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(state.cells_destroyed, 1, "cells_destroyed should be 1");

    // Per the spec, StagedEffects should be freshly inserted
    let staged = world.get::<StagedEffects>(entity);
    assert!(
        staged.is_some(),
        "StagedEffects should be inserted when absent and non-Do node is selected"
    );
    let staged = staged.unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "1 non-Do entry should be in StagedEffects"
    );
    assert_eq!(staged.0[0].0, "", "chip name should be empty string");
    assert_eq!(staged.0[0].1, non_do_node, "non-Do node should be pushed");
}

#[test]
fn fire_inserts_both_state_and_staged_effects_when_both_absent() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    // Entity has NEITHER EntropyEngineState NOR StagedEffects
    let entity = world.spawn_empty().id();
    let pool = vec![(1.0, non_do_node)];

    fire(entity, 3, &pool, "", &mut world);

    assert!(
        world.get::<EntropyEngineState>(entity).is_some(),
        "EntropyEngineState should be inserted"
    );
    assert!(
        world.get::<StagedEffects>(entity).is_some(),
        "StagedEffects should be inserted"
    );
    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(state.cells_destroyed, 1);
}
