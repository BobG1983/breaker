//! Tests for `fire()` core behavior: state insertion, `cells_destroyed` increment,
//! N-effects firing via `min(cells_destroyed, max_effects)`, and capping at `max_effects`.

use bevy::prelude::*;

use super::super::effect::*;
use crate::{
    effect::{
        EffectNode,
        core::{EffectKind, StagedEffects},
        effects::damage_boost::ActiveDamageBoosts,
    },
    shared::rng::GameRng,
};

// -- Behavior 9: fire() with no prior state inserts state and fires 1 effect --

#[test]
fn fire_inserts_state_and_fires_one_effect_when_no_prior_state() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 3, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 1,
        "cells_destroyed should be 1 (inserted at 0, then saturating_add(1))"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![2.0],
        "1 effect should be fired (cells_destroyed=1, min(1, 3)=1)"
    );
}

#[test]
fn fire_inserts_state_fresh_when_entity_has_none() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();

    // Verify no EntropyEngineState before fire
    assert!(
        world.get::<EntropyEngineState>(entity).is_none(),
        "entity should start without EntropyEngineState"
    );

    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];
    fire(entity, 3, &pool, "", &mut world);

    assert!(
        world.get::<EntropyEngineState>(entity).is_some(),
        "EntropyEngineState should be inserted by fire()"
    );
}

// -- Behavior 10: fire() increments cells_destroyed and fires N=min(cells_destroyed, max_effects) --

#[test]
fn fire_increments_cells_destroyed_and_fires_n_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 2 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 5, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 3,
        "cells_destroyed should be 3 (2 + 1)"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0.len(),
        3,
        "3 effects should fire (min(3, 5)=3), got {:?}",
        active.0
    );
    // All entries should be 2.0 since there's only one pool entry
    for val in &active.0 {
        assert!(
            (*val - 2.0).abs() < f32::EPSILON,
            "each entry should be 2.0, got {val}"
        );
    }
}

#[test]
fn fire_with_cells_destroyed_4_max_5_fires_5_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 4 },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 5, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 5,
        "cells_destroyed should be 5 (4 + 1)"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(active.0.len(), 5, "5 effects should fire (min(5, 5)=5)");
}

// -- Behavior 11: fire() caps at max_effects when cells_destroyed exceeds it --

#[test]
fn fire_caps_at_max_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState {
                cells_destroyed: 10,
            },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 3, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 11,
        "cells_destroyed should be 11 (10 + 1)"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0.len(),
        3,
        "effects capped at max_effects=3 despite cells_destroyed=11"
    );
}

// -- Behavior 12: fire() with max_effects=1 always fires exactly 1 --

#[test]
fn fire_with_max_effects_1_fires_exactly_one() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState {
                cells_destroyed: 50,
            },
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
        ))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 1, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 51,
        "cells_destroyed should be 51 (50 + 1)"
    );
    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0.len(),
        1,
        "exactly 1 effect should fire with max_effects=1"
    );
}

// -- Behavior 13: fire() with empty pool still increments cells_destroyed --

#[test]
fn fire_with_empty_pool_increments_cells_destroyed_but_fires_nothing() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
            EntropyEngineState { cells_destroyed: 5 },
            StagedEffects::default(),
        ))
        .id();
    let pool: Vec<(f32, EffectNode)> = vec![];

    fire(entity, 3, &pool, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 6,
        "cells_destroyed should increment to 6 even with empty pool"
    );
}
