//! Tests for `fire()` weighted random selection: single element, multi-element, and weight behavior.

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    effect::{
        core::{EffectKind, EffectNode, StagedEffects},
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
    shared::rng::GameRng,
};

// -- Behavior 1: fire() with single-element pool selects and fires that element --

#[test]
fn fire_with_single_element_pool_fires_that_effect() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, &pool, "", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![2.0],
        "single-element pool should fire DamageBoost(2.0), got {:?}",
        active.0
    );
}

#[test]
fn fire_with_single_element_pool_and_tiny_weight_selects_only_element() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();
    let pool = vec![(0.001, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, &pool, "", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![2.0],
        "single-element pool with weight 0.001 should still fire the only element"
    );
}

// -- Behavior 2: fire() with multi-element pool uses weighted random selection --

#[test]
fn fire_with_equal_weight_pool_selects_exactly_one_effect() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((
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

    fire(entity, &pool, "", &mut world);

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    let total = damage.0.len() + speed.0.len();
    assert_eq!(
        total, 1,
        "exactly one effect should be fired from two-element equal-weight pool, got {} total (damage: {:?}, speed: {:?})",
        total, damage.0, speed.0
    );
}

// -- Behavior 3: fire() weighted selection respects relative weights --

#[test]
fn fire_with_zero_weight_item_never_selects_it() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(0));
    let entity = world
        .spawn((ActiveDamageBoosts(vec![]), ActiveSpeedBoosts(vec![])))
        .id();
    let pool = vec![
        (100.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
        (
            0.0,
            EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
        ),
    ];

    fire(entity, &pool, "", &mut world);

    let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
    let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
    assert_eq!(
        damage.0,
        vec![2.0],
        "DamageBoost should always be selected when other weight is 0.0"
    );
    assert!(
        speed.0.is_empty(),
        "SpeedBoost with weight 0.0 should never be selected"
    );
}
