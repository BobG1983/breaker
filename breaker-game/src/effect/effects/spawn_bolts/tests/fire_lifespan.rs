//! Tests for `fire()` with lifespan timer behavior.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltLifespan, ExtraBolt},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    shared::rng::GameRng,
};

fn world_with_bolt_registry() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        },
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world
}

#[test]
fn fire_with_lifespan_adds_bolt_lifespan_timer() {
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, Some(5.0), false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let lifespan = world
        .get::<BoltLifespan>(bolt)
        .expect("bolt should have BoltLifespan with lifespan=Some(5.0)");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 5.0).abs() < 0.01,
        "BoltLifespan duration should be 5.0s, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn fire_with_very_short_lifespan_creates_valid_timer() {
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, Some(0.01), false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let lifespan = world
        .get::<BoltLifespan>(bolt)
        .expect("bolt should have BoltLifespan with lifespan=Some(0.01)");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 0.01).abs() < 0.001,
        "BoltLifespan should work with short duration 0.01, got {}",
        lifespan.0.duration().as_secs_f32()
    );
}

#[test]
fn fire_with_no_lifespan_does_not_add_bolt_lifespan() {
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    assert!(
        world.get::<BoltLifespan>(bolt).is_none(),
        "bolt should NOT have BoltLifespan when lifespan=None"
    );
}
