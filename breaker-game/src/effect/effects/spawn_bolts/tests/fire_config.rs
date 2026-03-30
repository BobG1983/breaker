//! Tests for `fire()` with custom `BoltConfig` values and velocity direction diversity.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    shared::rng::GameRng,
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

#[test]
fn fire_spawns_bolt_with_custom_base_speed() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        base_speed: 600.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "velocity magnitude should be custom base_speed (600.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_spawns_multiple_bolts_with_distinct_velocity_directions() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 3, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let velocities: Vec<Vec2> = query.iter(&world).map(|v| v.0).collect();
    assert_eq!(velocities.len(), 3, "should spawn 3 bolts");

    // All should have base_speed magnitude
    for vel in &velocities {
        assert!(
            (vel.length() - 400.0).abs() < 1.0,
            "each bolt velocity magnitude should be ~400.0, got {}",
            vel.length()
        );
    }

    // Probabilistically, at least two should differ in direction
    // (since each uses a separate random angle)
    let directions_differ = velocities[0].normalize() != velocities[1].normalize()
        || velocities[1].normalize() != velocities[2].normalize();
    assert!(
        directions_differ,
        "with 3 bolts and independent random angles, at least two should have different directions"
    );
}

#[test]
fn fire_uses_custom_radius_from_bolt_config() {
    let mut world = World::new();
    world.insert_resource(BoltConfig {
        radius: 6.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let scale = world
        .get::<Scale2D>(bolt)
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON,
        "Scale2D.x should use custom radius (6.0)"
    );
    assert!(
        (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D.y should use custom radius (6.0)"
    );

    let aabb = world.get::<Aabb2D>(bolt).expect("bolt should have Aabb2D");
    assert_eq!(aabb.half_extents, Vec2::new(6.0, 6.0));

    let radius = world
        .get::<BoltRadius>(bolt)
        .expect("bolt should have BoltRadius");
    assert!((radius.0 - 6.0).abs() < f32::EPSILON);
}
