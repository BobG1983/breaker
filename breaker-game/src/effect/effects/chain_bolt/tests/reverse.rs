//! Tests for `reverse()` chain bolt despawn behavior.

use bevy::prelude::*;
use rantzsoft_physics2d::constraint::DistanceConstraint;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{definition::BoltDefinition, registry::BoltRegistry},
    effect::effects::chain_bolt::effect::*,
    shared::rng::GameRng,
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
fn reverse_despawns_chain_bolt_and_constraint_and_removes_anchor_marker() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn((Position2D(Vec2::ZERO), ChainBoltAnchor)).id();

    // Manually spawn a chain bolt with a constraint entity
    let constraint_entity = world
        .spawn(DistanceConstraint {
            entity_a: anchor,
            entity_b: Entity::PLACEHOLDER, // will be replaced
            max_distance: 150.0,
        })
        .id();

    let chain_bolt = world
        .spawn((
            ChainBoltMarker(anchor),
            ChainBoltConstraint(constraint_entity),
        ))
        .id();

    // Fix constraint to reference the actual chain bolt
    world
        .get_mut::<DistanceConstraint>(constraint_entity)
        .unwrap()
        .entity_b = chain_bolt;

    reverse(anchor, 150.0, "", &mut world);

    // Chain bolt should be despawned
    assert!(
        world.get_entity(chain_bolt).is_err(),
        "chain bolt should be despawned"
    );

    // Constraint entity should be despawned
    assert!(
        world.get_entity(constraint_entity).is_err(),
        "constraint entity should be despawned"
    );

    // Anchor marker should be removed
    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_none(),
        "ChainBoltAnchor should be removed from anchor"
    );
}

#[test]
fn reverse_despawns_multiple_chain_bolts_and_constraints() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn((Position2D(Vec2::ZERO), ChainBoltAnchor)).id();

    // Spawn two chain bolts with their constraint entities
    let constraint_a = world
        .spawn(DistanceConstraint {
            entity_a: anchor,
            entity_b: Entity::PLACEHOLDER,
            max_distance: 150.0,
        })
        .id();
    let chain_bolt_a = world
        .spawn((ChainBoltMarker(anchor), ChainBoltConstraint(constraint_a)))
        .id();

    let constraint_b = world
        .spawn(DistanceConstraint {
            entity_a: anchor,
            entity_b: Entity::PLACEHOLDER,
            max_distance: 200.0,
        })
        .id();
    let chain_bolt_b = world
        .spawn((ChainBoltMarker(anchor), ChainBoltConstraint(constraint_b)))
        .id();

    reverse(anchor, 150.0, "", &mut world);

    // Both chain bolts should be despawned
    assert!(
        world.get_entity(chain_bolt_a).is_err(),
        "chain bolt A should be despawned"
    );
    assert!(
        world.get_entity(chain_bolt_b).is_err(),
        "chain bolt B should be despawned"
    );

    // Both constraints should be despawned
    assert!(
        world.get_entity(constraint_a).is_err(),
        "constraint A should be despawned"
    );
    assert!(
        world.get_entity(constraint_b).is_err(),
        "constraint B should be despawned"
    );

    // Anchor marker removed
    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_none(),
        "ChainBoltAnchor should be removed"
    );
}

#[test]
fn reverse_when_no_chain_bolts_is_noop() {
    let mut world = world_with_bolt_registry();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    // reverse with no chain bolts and no anchor marker should not panic.
    reverse(anchor, 150.0, "", &mut world);

    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_none(),
        "anchor should remain without ChainBoltAnchor"
    );
}
