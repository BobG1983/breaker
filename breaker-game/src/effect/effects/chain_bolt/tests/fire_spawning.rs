//! Tests for `fire()` core chain bolt spawning and physics component setup.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef, BoltRadius},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER, rng::GameRng},
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
    world
}

#[test]
fn fire_spawns_one_chain_bolt_with_full_physics() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

    fire(anchor, 150.0, "", &mut world);

    // Should spawn exactly ONE chain bolt entity.
    let chain_bolts: Vec<(Entity, &ChainBoltMarker)> = world
        .query::<(Entity, &ChainBoltMarker)>()
        .iter(&world)
        .collect();
    assert_eq!(
        chain_bolts.len(),
        1,
        "fire should spawn exactly one chain bolt"
    );

    let (chain_bolt_entity, marker) = chain_bolts[0];
    assert_eq!(marker.0, anchor, "chain bolt should reference the anchor");

    // Bolt marker
    assert!(
        world.get::<Bolt>(chain_bolt_entity).is_some(),
        "chain bolt should have Bolt component"
    );

    // Position2D from anchor
    let pos = world
        .get::<Position2D>(chain_bolt_entity)
        .expect("chain bolt should have Position2D");
    assert_eq!(
        pos.0,
        Vec2::new(100.0, 200.0),
        "chain bolt Position2D should match anchor position"
    );

    // Velocity2D — magnitude at base_speed (direction is random via GameRng)
    let vel = world
        .get::<Velocity2D>(chain_bolt_entity)
        .expect("chain bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "chain bolt velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );

    // Scale2D
    let scale = world
        .get::<Scale2D>(chain_bolt_entity)
        .expect("chain bolt should have Scale2D");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "Scale2D.x should be radius (8.0)"
    );
    assert!(
        (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D.y should be radius (8.0)"
    );

    // Aabb2D
    let aabb = world
        .get::<Aabb2D>(chain_bolt_entity)
        .expect("chain bolt should have Aabb2D");
    assert_eq!(aabb.center, Vec2::ZERO);
    assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

    // CollisionLayers
    let layers = world
        .get::<CollisionLayers>(chain_bolt_entity)
        .expect("chain bolt should have CollisionLayers");
    assert_eq!(layers.membership, BOLT_LAYER);
    assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

    // Anchor should have ChainBoltAnchor
    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_some(),
        "anchor should have ChainBoltAnchor component"
    );
}

// ── Behavior 5: fire() reads BoltDefinitionRef from anchor entity for chain bolt construction ──

#[test]
fn fire_reads_bolt_definition_ref_from_anchor_entity() {
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

    let anchor = world
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            BoltDefinitionRef("Heavy".to_string()),
        ))
        .id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt_entity = query.iter(&world).next().expect("chain bolt should exist");

    let vel = world
        .get::<Velocity2D>(chain_bolt_entity)
        .expect("chain bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "chain bolt velocity magnitude should be ~600.0 from Heavy definition, got {}",
        vel.0.length()
    );

    let scale = world
        .get::<Scale2D>(chain_bolt_entity)
        .expect("chain bolt should have Scale2D");
    assert!(
        (scale.x - 12.0).abs() < f32::EPSILON,
        "Scale2D.x should be 12.0 from Heavy definition, got {}",
        scale.x
    );

    let radius = world
        .get::<BoltRadius>(chain_bolt_entity)
        .expect("chain bolt should have BoltRadius");
    assert!(
        (radius.0 - 12.0).abs() < f32::EPSILON,
        "BoltRadius should be 12.0 from Heavy definition, got {}",
        radius.0
    );

    let base_speed = world
        .get::<rantzsoft_spatial2d::components::BaseSpeed>(chain_bolt_entity)
        .expect("chain bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 600.0).abs() < f32::EPSILON,
        "BaseSpeed should be 600.0 from Heavy definition, got {}",
        base_speed.0
    );
}

// ── Behavior 6: fire() falls back to "Bolt" default definition when anchor has no BoltDefinitionRef ──

#[test]
fn fire_falls_back_to_bolt_default_definition_when_anchor_has_no_definition_ref() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        make_bolt_definition("Bolt", 720.0, 14.0),
    );
    world.insert_resource(registry);
    world.insert_resource(GameRng::default());

    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt_entity = query.iter(&world).next().expect("chain bolt should exist");

    let vel = world
        .get::<Velocity2D>(chain_bolt_entity)
        .expect("chain bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 720.0).abs() < 1.0,
        "chain bolt velocity magnitude should be ~720.0 from Bolt default definition, got {}",
        vel.0.length()
    );

    let radius = world
        .get::<BoltRadius>(chain_bolt_entity)
        .expect("chain bolt should have BoltRadius");
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0 from Bolt default definition, got {}",
        radius.0
    );
}
