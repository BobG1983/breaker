//! Tests for `fire()` with `BoltRegistry`/`BoltDefinitionRef` values and velocity direction diversity.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinSpeed, Position2D, Scale2D, Velocity2D,
};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef, BoltRadius, ExtraBolt},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    shared::rng::GameRng,
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
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    world
}

// ── Behavior 1: fire() reads BoltDefinitionRef from source entity and uses BoltRegistry ──

#[test]
fn fire_reads_bolt_definition_ref_from_source_entity_and_uses_bolt_registry() {
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

    let entity = world
        .spawn((
            Position2D(Vec2::new(50.0, 100.0)),
            BoltDefinitionRef("Heavy".to_string()),
        ))
        .id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let vel = world
        .get::<Velocity2D>(bolt)
        .expect("bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "velocity magnitude should be ~600.0 from Heavy definition, got {}",
        vel.0.length()
    );

    let scale = world
        .get::<Scale2D>(bolt)
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 12.0).abs() < f32::EPSILON,
        "Scale2D.x should be 12.0 from Heavy definition, got {}",
        scale.x
    );
    assert!(
        (scale.y - 12.0).abs() < f32::EPSILON,
        "Scale2D.y should be 12.0 from Heavy definition, got {}",
        scale.y
    );

    let aabb = world.get::<Aabb2D>(bolt).expect("bolt should have Aabb2D");
    assert_eq!(aabb.half_extents, Vec2::new(12.0, 12.0));

    let radius = world
        .get::<BoltRadius>(bolt)
        .expect("bolt should have BoltRadius");
    assert!((radius.0 - 12.0).abs() < f32::EPSILON);

    let base_speed = world
        .get::<BaseSpeed>(bolt)
        .expect("bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 600.0).abs() < f32::EPSILON,
        "BaseSpeed should be 600.0 from Heavy definition, got {}",
        base_speed.0
    );

    let min_speed = world
        .get::<MinSpeed>(bolt)
        .expect("bolt should have MinSpeed");
    assert!(
        (min_speed.0 - 300.0).abs() < f32::EPSILON,
        "MinSpeed should be 300.0 from Heavy definition, got {}",
        min_speed.0
    );

    let max_speed = world
        .get::<MaxSpeed>(bolt)
        .expect("bolt should have MaxSpeed");
    assert!(
        (max_speed.0 - 1200.0).abs() < f32::EPSILON,
        "MaxSpeed should be 1200.0 from Heavy definition, got {}",
        max_speed.0
    );
}

// ── Behavior 1 edge case: velocity magnitude comes from definition ──

#[test]
fn fire_uses_definition_base_speed_not_bolt_config_default() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Heavy".to_string(),
        make_bolt_definition("Heavy", 600.0, 12.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());

    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            BoltDefinitionRef("Heavy".to_string()),
        ))
        .id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>();
    let vel = query.iter(&world).next().expect("bolt should exist");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "velocity magnitude should be ~600.0 (from definition base_speed), got {}",
        vel.0.length()
    );
}

// ── Behavior 2: fire() falls back to "Bolt" default definition when source has no BoltDefinitionRef ──

#[test]
fn fire_falls_back_to_bolt_default_definition_when_no_definition_ref() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());

    let entity = world.spawn(Position2D(Vec2::new(50.0, 100.0))).id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolt = query.iter(&world).next().expect("bolt should exist");

    let vel = world
        .get::<Velocity2D>(bolt)
        .expect("bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 720.0).abs() < 1.0,
        "velocity magnitude should be ~720.0 from Bolt default definition, got {}",
        vel.0.length()
    );

    let radius = world
        .get::<BoltRadius>(bolt)
        .expect("bolt should have BoltRadius");
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0 from Bolt default definition, got {}",
        radius.0
    );

    let base_speed = world
        .get::<BaseSpeed>(bolt)
        .expect("bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0 from Bolt default definition, got {}",
        base_speed.0
    );
}

// ── Behavior 3: fire() uses BoltRegistry for bolt spawning ──

#[test]
fn fire_does_not_require_bolt_config_when_bolt_registry_available() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            BoltDefinitionRef("Bolt".to_string()),
        ))
        .id();

    fire(entity, 1, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 1, "should spawn 1 bolt from BoltRegistry");
}

// ── Behavior 4: fire() spawns multiple bolts all using the same BoltDefinition ──

#[test]
fn fire_spawns_multiple_bolts_all_using_same_definition() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Heavy".to_string(),
        make_bolt_definition("Heavy", 600.0, 12.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());

    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            BoltDefinitionRef("Heavy".to_string()),
        ))
        .id();

    fire(entity, 3, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let bolts: Vec<Entity> = query.iter(&world).collect();
    assert_eq!(bolts.len(), 3, "should spawn 3 bolts");

    for bolt in &bolts {
        let vel = world
            .get::<Velocity2D>(*bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 600.0).abs() < 1.0,
            "each bolt velocity magnitude should be ~600.0, got {}",
            vel.0.length()
        );

        let radius = world
            .get::<BoltRadius>(*bolt)
            .expect("bolt should have BoltRadius");
        assert!(
            (radius.0 - 12.0).abs() < f32::EPSILON,
            "each bolt should have BoltRadius 12.0, got {}",
            radius.0
        );
    }
}

// ── Behavior 4 edge case: count=0 spawns no bolts ──

#[test]
fn fire_count_zero_with_bolt_registry_spawns_no_bolts() {
    let mut world = world_with_bolt_registry();
    let entity = world
        .spawn((
            Position2D(Vec2::ZERO),
            BoltDefinitionRef("Bolt".to_string()),
        ))
        .id();

    fire(entity, 0, None, false, "", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "count=0 should spawn zero bolts");
}
