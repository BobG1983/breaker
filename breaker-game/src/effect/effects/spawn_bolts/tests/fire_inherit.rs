//! Tests for `fire()` effect inheritance (`BoundEffects` / `StagedEffects` copying).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        resources::BoltConfig,
    },
    effect::{BoundEffects, EffectKind, EffectNode, StagedEffects},
    shared::rng::GameRng,
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

#[test]
fn fire_with_inherit_true_copies_bound_effects() {
    let mut world = world_with_bolt_config();
    let bound = BoundEffects(vec![(
        "test_chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
    )]);
    let entity = world
        .spawn((Position2D(Vec2::ZERO), bound, StagedEffects::default()))
        .id();

    fire(entity, 2, None, true, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(bolts.len(), 2, "should spawn 2 bolts");

    for bolt in &bolts {
        let effects = world
            .get::<BoundEffects>(*bolt)
            .expect("spawned bolt should have BoundEffects when inherit=true");
        assert_eq!(
            effects.0.len(),
            1,
            "BoundEffects should have 1 entry, got {}",
            effects.0.len()
        );
        assert_eq!(effects.0[0].0, "test_chip");
    }
}

#[test]
fn fire_with_inherit_true_and_empty_bound_effects_spawns_empty() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((Position2D(Vec2::ZERO), BoundEffects::default()))
        .id();

    fire(entity, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either has BoundEffects with empty vec, or no BoundEffects at all — both are acceptable
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "empty BoundEffects should produce empty BoundEffects on spawned bolt"
        );
    }
}

#[test]
fn fire_with_inherit_false_does_not_copy_bound_effects() {
    let mut world = world_with_bolt_config();
    let bound = BoundEffects(vec![(
        "chip".to_string(),
        EffectNode::Do(EffectKind::DamageBoost(1.5)),
    )]);
    let entity = world.spawn((Position2D(Vec2::ZERO), bound)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let effects = world.get::<BoundEffects>(bolt);
    // Either no BoundEffects at all, or empty BoundEffects
    if let Some(effects) = effects {
        assert!(
            effects.0.is_empty(),
            "inherit=false should not copy BoundEffects to spawned bolt"
        );
    }
}

#[test]
fn fire_with_inherit_true_and_no_bound_effects_does_not_panic() {
    let mut world = world_with_bolt_config();
    // Entity has Position2D but no BoundEffects component
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    // Should not panic
    fire(entity, 1, None, true, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 1, "bolt should still be spawned");
}
