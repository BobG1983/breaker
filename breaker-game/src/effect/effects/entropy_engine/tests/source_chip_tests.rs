//! Tests for `fire()` forwarding `source_chip` to inner `Do` effects and
//! `StagedEffects` pushes.

use bevy::prelude::*;

use crate::{
    effect::{
        EffectNode,
        core::{EffectKind, StagedEffects, Trigger},
        effects::{damage_boost::ActiveDamageBoosts, entropy_engine::effect::*},
    },
    shared::rng::GameRng,
};

// ── Section N: meta-effect forwards source_chip ──

#[test]
fn fire_forwards_source_chip_to_inner_do_effects() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, 3, &pool, "entropy_chip", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert!(
        !active.0.is_empty(),
        "inner effects should fire — proves source_chip was threaded through"
    );
}

#[test]
fn fire_forwards_source_chip_to_staged_effects_push() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn(StagedEffects::default()).id();
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let pool = vec![(1.0, non_do_node)];

    fire(entity, 3, &pool, "entropy_chip", &mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert!(!staged.0.is_empty(), "StagedEffects should have entries");

    for entry in &staged.0 {
        assert_eq!(
            entry.0, "entropy_chip",
            "StagedEffects entry should have chip_name 'entropy_chip' forwarded from source_chip, not empty string"
        );
    }
}

#[test]
fn fire_forwards_empty_source_chip_to_staged_effects_push() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world.spawn(StagedEffects::default()).id();
    let non_do_node = EffectNode::When {
        trigger: Trigger::CellDestroyed,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let pool = vec![(1.0, non_do_node)];

    fire(entity, 3, &pool, "", &mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert!(!staged.0.is_empty());

    for entry in &staged.0 {
        assert_eq!(
            entry.0, "",
            "empty source_chip should forward as empty chip_name"
        );
    }
}
