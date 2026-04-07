//! Tests for `reverse()` no-op behavior and `source_chip` forwarding through `fire()`.

use bevy::prelude::*;

use crate::{
    effect::{
        core::{EffectKind, EffectNode, StagedEffects, Trigger},
        effects::{damage_boost::ActiveDamageBoosts, random_effect::system::*},
    },
    shared::rng::GameRng,
};

// -- Behavior 8: reverse() is a no-op --

#[test]
fn reverse_preserves_existing_state() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![2.0])).id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    reverse(entity, &pool, "", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![2.0],
        "reverse should not modify ActiveDamageBoosts"
    );
}

#[test]
fn reverse_on_entity_with_no_components_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    // Should not panic
    reverse(entity, &pool, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

// -- Section N: meta-effect forwards source_chip --

#[test]
fn fire_forwards_source_chip_to_inner_do_effect() {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    let entity = world
        .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
        .id();
    let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

    fire(entity, &pool, "chaos_chip", &mut world);

    let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        active.0,
        vec![2.0],
        "inner effect should fire — proves source_chip was threaded through"
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

    fire(entity, &pool, "chaos_chip", &mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0].0, "chaos_chip",
        "StagedEffects entry should have chip_name 'chaos_chip' forwarded from source_chip, not empty string"
    );
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

    fire(entity, &pool, "", &mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    assert_eq!(
        staged.0[0].0, "",
        "empty source_chip should forward as empty chip_name"
    );
}
