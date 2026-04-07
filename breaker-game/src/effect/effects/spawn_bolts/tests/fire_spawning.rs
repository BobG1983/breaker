//! Tests for `fire()` core bolt spawning and physics component setup.

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltRadius, ExtraBolt},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    effect::effects::spawn_bolts::effect::*,
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER, rng::GameRng},
    state::types::{NodeState, RunState},
};

fn world_with_bolt_registry() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
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
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world
}

#[test]
fn fire_spawns_requested_count_with_full_physics_components() {
    let mut world = world_with_bolt_registry();
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
            .get::<BaseSpeed>(*bolt)
            .expect("bolt should have BaseSpeed");
        assert!((base.0 - 400.0).abs() < f32::EPSILON);

        let min = world
            .get::<MinSpeed>(*bolt)
            .expect("bolt should have MinSpeed");
        assert!((min.0 - 200.0).abs() < f32::EPSILON);

        let max = world
            .get::<MaxSpeed>(*bolt)
            .expect("bolt should have MaxSpeed");
        assert!((max.0 - 800.0).abs() < f32::EPSILON);

        let radius = world
            .get::<BoltRadius>(*bolt)
            .expect("bolt should have BoltRadius");
        assert!((radius.0 - 8.0).abs() < f32::EPSILON);

        // CleanupOnExit<NodeState>
        assert!(
            world.get::<CleanupOnExit<NodeState>>(*bolt).is_some(),
            "bolt should have CleanupOnExit<NodeState>"
        );

        // Visual components: rendered extra bolts have Mesh2d, MeshMaterial2d, and GameDrawLayer::Bolt
        assert!(
            matches!(world.get::<GameDrawLayer>(*bolt), Some(GameDrawLayer::Bolt)),
            "rendered extra bolt should have GameDrawLayer::Bolt"
        );
        assert!(
            world.get::<Mesh2d>(*bolt).is_some(),
            "rendered extra bolt should have Mesh2d"
        );
        assert!(
            world.get::<MeshMaterial2d<ColorMaterial>>(*bolt).is_some(),
            "rendered extra bolt should have MeshMaterial2d<ColorMaterial>"
        );
    }
}

#[test]
fn fire_count_one_spawns_exactly_one_bolt() {
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 0, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "count=0 should spawn zero bolts, got {count}");
}

#[test]
fn fire_spawns_bolts_with_randomized_velocity_at_base_speed() {
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
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
    let mut world = world_with_bolt_registry();
    let entity = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    assert!(
        world.get::<ExtraBolt>(bolt).is_some(),
        "bolt should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(bolt).is_some(),
        "bolt should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(bolt).is_none(),
        "bolt should NOT have CleanupOnExit<RunState>"
    );
}
