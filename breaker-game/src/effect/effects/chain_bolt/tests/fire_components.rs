//! Tests for `fire()` chain bolt component setup: `ExtraBolt`, `DistanceConstraint`,
//! `ChainBoltAnchor`, `BoltDefinition`, and `Position2D` source.

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::constraint::DistanceConstraint;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::{components::ExtraBolt, definition::BoltDefinition, registry::BoltRegistry},
    effect::effects::chain_bolt::effect::*,
    shared::rng::GameRng,
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
fn fire_spawns_chain_bolt_with_extra_bolt_and_cleanup_on_node_exit() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = query.iter(&world).next().expect("chain bolt should exist");

    assert!(
        world.get::<ExtraBolt>(chain_bolt).is_some(),
        "chain bolt should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(chain_bolt).is_some(),
        "chain bolt should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(chain_bolt).is_none(),
        "chain bolt should NOT have CleanupOnExit<RunState>"
    );
}

#[test]
fn fire_spawns_distance_constraint_entity() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    // Find the chain bolt entity
    let mut chain_query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = chain_query
        .iter(&world)
        .next()
        .expect("chain bolt should exist");

    // Find the DistanceConstraint entity
    let mut constraint_query = world.query::<(Entity, &DistanceConstraint)>();
    let constraints: Vec<_> = constraint_query.iter(&world).collect();
    assert_eq!(
        constraints.len(),
        1,
        "fire should spawn exactly one DistanceConstraint entity"
    );

    let (constraint_entity, constraint) = constraints[0];
    assert_eq!(
        constraint.entity_a, anchor,
        "DistanceConstraint entity_a should be the anchor"
    );
    assert_eq!(
        constraint.entity_b, chain_bolt,
        "DistanceConstraint entity_b should be the chain bolt"
    );
    assert!(
        (constraint.max_distance - 150.0).abs() < f32::EPSILON,
        "DistanceConstraint max_distance should be 150.0, got {}",
        constraint.max_distance
    );

    // Constraint entity should be distinct from both anchor and chain bolt
    assert_ne!(
        constraint_entity, anchor,
        "constraint entity should not be the anchor"
    );
    assert_ne!(
        constraint_entity, chain_bolt,
        "constraint entity should not be the chain bolt"
    );
}

#[test]
fn fire_stores_constraint_reference_on_chain_bolt() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    // Find chain bolt
    let mut chain_query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = chain_query
        .iter(&world)
        .next()
        .expect("chain bolt should exist");

    // Chain bolt should have ChainBoltConstraint
    let constraint_ref = world
        .get::<ChainBoltConstraint>(chain_bolt)
        .expect("chain bolt should have ChainBoltConstraint");

    // The stored entity should actually have a DistanceConstraint
    let constraint = world
        .get::<DistanceConstraint>(constraint_ref.0)
        .expect("ChainBoltConstraint should reference a valid DistanceConstraint entity");
    assert_eq!(constraint.entity_a, anchor);
    assert_eq!(constraint.entity_b, chain_bolt);
}

#[test]
fn fire_chain_bolt_velocity_magnitude_at_base_speed() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = query.iter(&world).next().expect("chain bolt should exist");

    let vel = world
        .get::<Velocity2D>(chain_bolt)
        .expect("chain bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 400.0).abs() < 1.0,
        "chain bolt velocity magnitude should be base_speed (400.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_chain_bolt_custom_base_speed() {
    let mut world = World::new();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 600.0,
            min_speed: 300.0,
            max_speed: 1200.0,
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

    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = query.iter(&world).next().expect("chain bolt should exist");

    let vel = world
        .get::<Velocity2D>(chain_bolt)
        .expect("chain bolt should have Velocity2D");
    assert!(
        (vel.0.length() - 600.0).abs() < 1.0,
        "chain bolt velocity magnitude should use custom base_speed (600.0), got {}",
        vel.0.length()
    );
}

#[test]
fn fire_marks_anchor_with_chain_bolt_anchor() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_none(),
        "anchor should not have ChainBoltAnchor before fire"
    );

    fire(anchor, 150.0, "", &mut world);

    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_some(),
        "anchor should have ChainBoltAnchor after fire"
    );
}

#[test]
fn fire_anchor_already_has_chain_bolt_anchor_no_panic() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn((Position2D(Vec2::ZERO), ChainBoltAnchor)).id();

    // Should not panic when anchor already has ChainBoltAnchor
    fire(anchor, 150.0, "", &mut world);

    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_some(),
        "anchor should still have ChainBoltAnchor"
    );
}

#[test]
fn fire_zero_tether_distance_spawns_constraint() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 0.0, "", &mut world);

    let mut query = world.query::<&DistanceConstraint>();
    let constraints: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        constraints.len(),
        1,
        "zero tether distance should still spawn a DistanceConstraint"
    );
    assert!(
        (constraints[0].max_distance - 0.0).abs() < f32::EPSILON,
        "max_distance should be 0.0"
    );
}

#[test]
fn fire_reads_position_from_position2d_not_transform() {
    let mut world = world_with_bolt_registry();
    let anchor = world
        .spawn((
            Position2D(Vec2::new(50.0, 75.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ))
        .id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = query.iter(&world).next().expect("chain bolt should exist");

    let pos = world
        .get::<Position2D>(chain_bolt)
        .expect("chain bolt should have Position2D");
    assert_eq!(
        pos.0,
        Vec2::new(50.0, 75.0),
        "chain bolt should use Position2D (50, 75) not Transform (999, 999)"
    );
}
