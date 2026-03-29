use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, constraint::DistanceConstraint,
};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};

use super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd, WALL_LAYER,
        rng::GameRng,
    },
};

fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}

// -- fire tests ──────────────────────────────────────────────────

#[test]
fn fire_spawns_one_chain_bolt_with_full_physics() {
    let mut world = world_with_bolt_config();
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

#[test]
fn fire_spawns_chain_bolt_with_extra_bolt_and_cleanup_on_node_exit() {
    let mut world = world_with_bolt_config();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    fire(anchor, 150.0, "", &mut world);

    let mut query = world.query_filtered::<Entity, With<ChainBoltMarker>>();
    let chain_bolt = query.iter(&world).next().expect("chain bolt should exist");

    assert!(
        world.get::<ExtraBolt>(chain_bolt).is_some(),
        "chain bolt should have ExtraBolt"
    );
    assert!(
        world.get::<CleanupOnNodeExit>(chain_bolt).is_some(),
        "chain bolt should have CleanupOnNodeExit"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(chain_bolt).is_none(),
        "chain bolt should NOT have CleanupOnRunEnd"
    );
}

#[test]
fn fire_spawns_distance_constraint_entity() {
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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
    world.insert_resource(BoltConfig {
        base_speed: 600.0,
        ..BoltConfig::default()
    });
    world.insert_resource(GameRng::default());

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
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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

// -- reverse tests ───────────────────────────────────────────────

#[test]
fn reverse_despawns_chain_bolt_and_constraint_and_removes_anchor_marker() {
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
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
    let mut world = world_with_bolt_config();
    let anchor = world.spawn(Position2D(Vec2::ZERO)).id();

    // reverse with no chain bolts and no anchor marker should not panic.
    reverse(anchor, 150.0, "", &mut world);

    assert!(
        world.get::<ChainBoltAnchor>(anchor).is_none(),
        "anchor should remain without ChainBoltAnchor"
    );
}
