//! Tests for spawned bolt physics components (via `spawn_extra_bolt`) and velocity override.

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef, BoltRadius, ExtraBolt, ImpactSide, LastImpact},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    effect::effects::mirror_protocol::effect::*,
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, GameDrawLayer, WALL_LAYER, rng::GameRng},
    state::types::NodeState,
};

fn make_bolt_definition(name: &str, base_speed: f32, radius: f32) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed,
        min_speed: base_speed / 2.0,
        max_speed: base_speed * 2.0,
        radius,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

fn world_with_bolt_registry() -> World {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert("Bolt".to_string(), make_bolt_definition("Bolt", 400.0, 8.0));
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world
}

// -- Behavior 13: Spawned bolts have ExtraBolt marker and full physics components --

#[test]
fn spawned_bolt_has_full_physics_components_from_spawn_extra_bolt() {
    let mut world = world_with_bolt_registry();
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

    // CleanupOnExit<NodeState>
    assert!(
        world.get::<CleanupOnExit<NodeState>>(bolt).is_some(),
        "should have CleanupOnExit<NodeState>"
    );

    // Visual components: rendered mirror bolts have Mesh2d, MeshMaterial2d, and GameDrawLayer::Bolt
    assert!(
        matches!(world.get::<GameDrawLayer>(bolt), Some(GameDrawLayer::Bolt)),
        "rendered mirror bolt should have GameDrawLayer::Bolt"
    );
    assert!(
        world.get::<Mesh2d>(bolt).is_some(),
        "rendered mirror bolt should have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(bolt).is_some(),
        "rendered mirror bolt should have MeshMaterial2d<ColorMaterial>"
    );
}

// -- Behavior 13 edge case: spawn_extra_bolt gives random velocity, but fire() overwrites --

#[test]
fn fire_overwrites_spawn_extra_bolt_random_velocity_with_mirror_velocity() {
    let mut world = world_with_bolt_registry();
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

// ── Behavior 7: fire() reads BoltDefinitionRef from source bolt for mirrored bolt construction ──

#[test]
fn fire_reads_bolt_definition_ref_from_source_bolt_for_mirrored_construction() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Heavy".to_string(),
        make_bolt_definition("Heavy", 600.0, 12.0),
    );
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();

    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 200.0),
                side: ImpactSide::Top,
            },
            BoltDefinitionRef("Heavy".to_string()),
        ))
        .id();

    fire(bolt_entity, false, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let mirrored = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");

    let radius = world
        .get::<BoltRadius>(mirrored)
        .expect("mirrored bolt should have BoltRadius");
    assert!(
        (radius.0 - 12.0).abs() < f32::EPSILON,
        "BoltRadius should be 12.0 from Heavy definition, got {}",
        radius.0
    );

    let scale = world
        .get::<Scale2D>(mirrored)
        .expect("mirrored bolt should have Scale2D");
    assert!(
        (scale.x - 12.0).abs() < f32::EPSILON,
        "Scale2D.x should be 12.0 from Heavy definition, got {}",
        scale.x
    );

    let base_speed = world
        .get::<BaseSpeed>(mirrored)
        .expect("mirrored bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 600.0).abs() < f32::EPSILON,
        "BaseSpeed should be 600.0 from Heavy definition, got {}",
        base_speed.0
    );
}

// ── Behavior 8: fire() falls back to "Bolt" default definition when source has no BoltDefinitionRef ──

#[test]
fn fire_falls_back_to_bolt_default_definition_when_source_bolt_has_no_definition_ref() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();

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

    fire(bolt_entity, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let mirrored = query
        .iter(&world)
        .next()
        .expect("mirrored bolt should exist");

    let radius = world
        .get::<BoltRadius>(mirrored)
        .expect("mirrored bolt should have BoltRadius");
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0 from Bolt default definition, got {}",
        radius.0
    );

    let base_speed = world
        .get::<BaseSpeed>(mirrored)
        .expect("mirrored bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0 from Bolt default definition, got {}",
        base_speed.0
    );
}
