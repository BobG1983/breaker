//! Tests for `fire()` core bolt spawning and physics component setup.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Scale2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, GameDrawLayer,
        WALL_LAYER, rng::GameRng,
    },
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

#[test]
fn fire_spawns_requested_count_with_full_physics_components() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 100.0))).id();

    fire(entity, 3, None, false, "", &mut world);

    // Query for spawned bolt entities (excluding the owner)
    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(
        bolts.len(),
        3,
        "expected 3 bolts spawned, got {}",
        bolts.len()
    );

    for bolt in &bolts {
        // Position2D from owner
        let pos = world
            .get::<Position2D>(*bolt)
            .expect("bolt should have Position2D");
        assert_eq!(
            pos.0,
            Vec2::new(50.0, 100.0),
            "bolt Position2D should match owner"
        );

        // PreviousPosition
        let prev = world
            .get::<PreviousPosition>(*bolt)
            .expect("bolt should have PreviousPosition");
        assert_eq!(prev.0, Vec2::new(50.0, 100.0));

        // Scale2D — radius=8.0 from default config
        let scale = world
            .get::<Scale2D>(*bolt)
            .expect("bolt should have Scale2D");
        assert!((scale.x - 8.0).abs() < f32::EPSILON);
        assert!((scale.y - 8.0).abs() < f32::EPSILON);

        // Aabb2D
        let aabb = world.get::<Aabb2D>(*bolt).expect("bolt should have Aabb2D");
        assert_eq!(aabb.center, Vec2::ZERO);
        assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

        // CollisionLayers
        let layers = world
            .get::<CollisionLayers>(*bolt)
            .expect("bolt should have CollisionLayers");
        assert_eq!(layers.membership, BOLT_LAYER);
        assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

        // Speed components
        let base = world
            .get::<BoltBaseSpeed>(*bolt)
            .expect("bolt should have BoltBaseSpeed");
        assert!((base.0 - 400.0).abs() < f32::EPSILON);

        let min = world
            .get::<BoltMinSpeed>(*bolt)
            .expect("bolt should have BoltMinSpeed");
        assert!((min.0 - 200.0).abs() < f32::EPSILON);

        let max = world
            .get::<BoltMaxSpeed>(*bolt)
            .expect("bolt should have BoltMaxSpeed");
        assert!((max.0 - 800.0).abs() < f32::EPSILON);

        let radius = world
            .get::<BoltRadius>(*bolt)
            .expect("bolt should have BoltRadius");
        assert!((radius.0 - 8.0).abs() < f32::EPSILON);

        // CleanupOnNodeExit
        assert!(
            world.get::<CleanupOnNodeExit>(*bolt).is_some(),
            "bolt should have CleanupOnNodeExit"
        );

        // GameDrawLayer::Bolt
        assert!(
            world.get::<GameDrawLayer>(*bolt).is_some(),
            "bolt should have GameDrawLayer"
        );
    }
}

#[test]
fn fire_count_one_spawns_exactly_one_bolt() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 1,
        "count=1 should spawn exactly one bolt, got {count}"
    );
}

#[test]
fn fire_count_zero_spawns_no_bolts() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 0, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "count=0 should spawn zero bolts, got {count}");
}

#[test]
fn fire_spawns_bolts_with_randomized_velocity_at_base_speed() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_spawns_bolt_at_owner_position2d_not_transform() {
    let mut world = world_with_bolt_config();
    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 75.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Position2D, (With<Bolt>, With<ExtraBolt>)>();
    let pos = query.iter(&world).next().expect("bolt should exist");
    assert_eq!(
        pos.0,
        Vec2::new(50.0, 75.0),
        "bolt should use Position2D (50, 75), not Transform (999, 999)"
    );
}

#[test]
fn fire_spawns_bolt_at_zero_when_owner_has_no_position2d() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn_empty().id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Position2D, (With<Bolt>, With<ExtraBolt>)>();
    let pos = query.iter(&world).next().expect("bolt should exist");
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "bolt should default to Vec2::ZERO when owner has no Position2D"
    );
}

#[test]
fn fire_marks_bolts_with_extra_bolt_and_cleanup_on_node_exit() {
    let mut world = world_with_bolt_config();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    assert!(
        world.get::<ExtraBolt>(bolt).is_some(),
        "bolt should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(bolt).is_some(),
        "bolt should have CleanupOnNodeExit"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(bolt).is_none(),
        "bolt should NOT have CleanupOnRunEnd"
    );
}
