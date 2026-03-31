//! Tests for spawned bolt physics components (via `spawn_extra_bolt`) and velocity override.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltRadius, ExtraBolt, ImpactSide, LastImpact},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer, WALL_LAYER,
        rng::GameRng,
    },
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

// -- Behavior 13: Spawned bolts have ExtraBolt marker and full physics components --

#[test]
fn spawned_bolt_has_full_physics_components_from_spawn_extra_bolt() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");

    // Bolt + ExtraBolt markers
    assert!(world.get::<Bolt>(bolt).is_some(), "should have Bolt");
    assert!(
        world.get::<ExtraBolt>(bolt).is_some(),
        "should have ExtraBolt"
    );

    // Position2D at mirror position: (2*50 - 60, 250) = (40, 250)
    let pos = world
        .get::<Position2D>(bolt)
        .expect("should have Position2D");
    assert_eq!(pos.0, Vec2::new(40.0, 250.0), "mirror position");

    // PreviousPosition
    assert!(
        world.get::<PreviousPosition>(bolt).is_some(),
        "should have PreviousPosition"
    );

    // Scale2D -- radius=8.0 from default config
    let scale = world.get::<Scale2D>(bolt).expect("should have Scale2D");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "Scale2D.x should be 8.0"
    );
    assert!(
        (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D.y should be 8.0"
    );

    // Aabb2D
    let aabb = world.get::<Aabb2D>(bolt).expect("should have Aabb2D");
    assert_eq!(aabb.center, Vec2::ZERO);
    assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

    // CollisionLayers
    let layers = world
        .get::<CollisionLayers>(bolt)
        .expect("should have CollisionLayers");
    assert_eq!(layers.membership, BOLT_LAYER);
    assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

    // Velocity2D at mirror velocity: (-100, 400)
    let vel = world
        .get::<Velocity2D>(bolt)
        .expect("should have Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "velocity should be mirror velocity, not random from spawn_extra_bolt"
    );

    // Speed components
    let base = world.get::<BaseSpeed>(bolt).expect("should have BaseSpeed");
    assert!((base.0 - 400.0).abs() < f32::EPSILON);

    let min = world.get::<MinSpeed>(bolt).expect("should have MinSpeed");
    assert!((min.0 - 200.0).abs() < f32::EPSILON);

    let max = world.get::<MaxSpeed>(bolt).expect("should have MaxSpeed");
    assert!((max.0 - 800.0).abs() < f32::EPSILON);

    let radius = world
        .get::<BoltRadius>(bolt)
        .expect("should have BoltRadius");
    assert!((radius.0 - 8.0).abs() < f32::EPSILON);

    // CleanupOnNodeExit
    assert!(
        world.get::<CleanupOnNodeExit>(bolt).is_some(),
        "should have CleanupOnNodeExit"
    );

    // GameDrawLayer::Bolt
    assert!(
        world.get::<GameDrawLayer>(bolt).is_some(),
        "should have GameDrawLayer"
    );
}

// -- Behavior 13 edge case: spawn_extra_bolt gives random velocity, but fire() overwrites --

#[test]
fn fire_overwrites_spawn_extra_bolt_random_velocity_with_mirror_velocity() {
    let mut world = world_with_bolt_config();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    // spawn_extra_bolt generates random velocity, but fire() must overwrite it
    assert_eq!(
        vel.0,
        Vec2::new(-100.0, 400.0),
        "velocity must be deterministic mirror value, not random from spawn_extra_bolt"
    );
}
