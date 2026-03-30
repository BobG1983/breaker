//! Tests for `fire()` edge cases: empty pool, non-`Do` node staging, deterministic selection,
//! and all-zero weights.

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    effect::{
        core::{EffectKind, EffectNode, StagedEffects, Trigger},
        effects::{
            bump_force::ActiveBumpForces, damage_boost::ActiveDamageBoosts,
            speed_boost::ActiveSpeedBoosts,
        },
    },
    shared::rng::GameRng,
};

// -- Behavior 4: fire() with empty pool is a no-op --

#[test]
fn fire_with_empty_pool_is_noop() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn_empty().id();
    let pool: Vec<(f32, EffectNode)> = vec![];

    // Should not panic; early return on empty pool.
    fire(entity, &pool, "", &mut world);

    // Entity should remain unchanged — no Active* components added
    assert!(
        world.get::<ActiveDamageBoosts>(entity).is_none(),
        "no ActiveDamageBoosts should be added on empty pool"
    );
}

#[test]
fn fire_with_empty_pool_preserves_existing_state() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn(ActiveDamageBoosts(vec![5.0])).id();
    let pool: Vec<(f32, EffectNode)> = vec![];

    fire(entity, &pool, "", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![5.0],
        "existing ActiveDamageBoosts should be preserved on empty pool"
    );
}

// -- Behavior 5: fire() with non-Do node pushes to StagedEffects --

#[test]
fn fire_with_non_do_node_pushes_to_staged_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn(StagedEffects::default()).id();
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let pool = vec![(1.0, non_do_node.clone())];

    fire(entity, &pool, "", &mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "one entry should be pushed to StagedEffects"
    );
    assert_eq!(
        staged.0[0].0, "",
        "chip name should be empty string for RandomEffect dispatch"
    );
    assert_eq!(
        staged.0[0].1, non_do_node,
        "the non-Do node should be pushed to StagedEffects"
    );
}

#[test]
fn fire_with_non_do_node_silently_drops_when_no_staged_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    // Entity WITHOUT StagedEffects component
    let entity = world.spawn_empty().id();
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let pool = vec![(1.0, non_do_node)];

    // Should not panic — silently dropped
    fire(entity, &pool, "", &mut world);

    assert!(
        world.get::<StagedEffects>(entity).is_none(),
        "StagedEffects should not be inserted if absent"
    );
}

// -- Behavior 6: fire() deterministic across separate worlds with same seed --

#[test]
fn fire_deterministic_across_separate_worlds_with_same_seed() {
    let pool = vec![
        (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            1.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
        (1.0, EffectNode::Do(EffectKind::BumpForce(50.0))),
    ];

    // World 1
    let mut world1 = World::new();
    world1.insert_resource(GameRng::from_seed(99));
    let entity1 = world1
        .spawn((
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
            ActiveBumpForces(vec![]),
        ))
        .id();
    fire(entity1, &pool, "", &mut world1);

    // World 2
    let mut world2 = World::new();
    world2.insert_resource(GameRng::from_seed(99));
    let entity2 = world2
        .spawn((
            StagedEffects::default(),
            ActiveDamageBoosts(vec![]),
            ActiveSpeedBoosts(vec![]),
            ActiveBumpForces(vec![]),
        ))
        .id();
    fire(entity2, &pool, "", &mut world2);

    let damage1 = world1.get::<ActiveDamageBoosts>(entity1).unwrap();
    let damage2 = world2.get::<ActiveDamageBoosts>(entity2).unwrap();
    let speed1 = world1.get::<ActiveSpeedBoosts>(entity1).unwrap();
    let speed2 = world2.get::<ActiveSpeedBoosts>(entity2).unwrap();
    let bump1 = world1.get::<ActiveBumpForces>(entity1).unwrap();
    let bump2 = world2.get::<ActiveBumpForces>(entity2).unwrap();

    assert_eq!(
        damage1.0, damage2.0,
        "ActiveDamageBoosts must match across worlds with same seed"
    );
    assert_eq!(
        speed1.0, speed2.0,
        "ActiveSpeedBoosts must match across worlds with same seed"
    );
    assert_eq!(
        bump1.0, bump2.0,
        "ActiveBumpForces must match across worlds with same seed"
    );
}

// -- Behavior 7: fire() with all-zero weights pool is a no-op --

#[test]
fn fire_with_all_zero_weights_is_noop() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((ActiveDamageBoosts(vec![]), ActiveSpeedBoosts(vec![])))
        .id();
    let pool = vec![
        (0.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            0.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ];

    fire(entity, &pool, "", &mut world);

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(
        damage.0.is_empty(),
        "no DamageBoost should be fired with all-zero weights"
    );
    assert!(
        speed.0.is_empty(),
        "no SpeedBoost should be fired with all-zero weights"
    );
}

#[test]
fn fire_with_single_zero_weight_is_noop() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();
    let pool = vec![(0.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, &pool, "", &mut world);

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        damage.0.is_empty(),
        "single-element pool with weight 0.0 should be a no-op (WeightedIndex fails)"
    );
}
