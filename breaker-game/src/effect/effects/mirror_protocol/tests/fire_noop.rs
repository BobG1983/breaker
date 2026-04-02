//! Tests for `fire()` no-op guard conditions (missing `Bolt`, missing `LastImpact`, despawned entity).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::effect::*;
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt, ImpactSide, LastImpact},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
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
    world
}

// -- Behavior 6: fire() on entity WITHOUT Bolt component is a noop --

#[test]
fn fire_on_entity_without_bolt_component_is_noop() {
    let mut world = world_with_bolt_registry();
    let entity = world
        .spawn((
            Position2D(Vec2::new(100.0, 200.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
            LastImpact {
                position: Vec2::new(50.0, 100.0),
                side: ImpactSide::Top,
            },
        ))
        .id();

    fire(entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 0,
        "no bolts should be spawned when entity lacks Bolt component"
    );
}

// -- Behavior 7: fire() on entity without LastImpact component is a noop --

#[test]
fn fire_on_bolt_without_last_impact_is_noop() {
    let mut world = world_with_bolt_registry();
    let bolt_entity = world
        .spawn((
            Bolt,
            Position2D(Vec2::new(60.0, 250.0)),
            Velocity2D(Vec2::new(100.0, 400.0)),
        ))
        .id();

    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 0,
        "no bolts should be spawned when bolt entity lacks LastImpact component"
    );
}

// -- Behavior 8: fire() on despawned entity is a noop --

#[test]
fn fire_on_despawned_entity_is_noop() {
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

    world.despawn(bolt_entity);

    // Should not panic
    fire(bolt_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 0,
        "no bolts should be spawned when entity is despawned"
    );
}

// -- Behavior 8 edge case: Entity ID was never valid --

#[test]
fn fire_on_never_valid_entity_id_is_noop() {
    let mut world = world_with_bolt_registry();
    let invalid_entity = Entity::from_raw_u32(9999).unwrap();

    // Should not panic
    fire(invalid_entity, true, "mirror_protocol", &mut world);

    let mut query = world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>();
    let count = query.iter(&world).count();
    assert_eq!(count, 0, "no bolts should be spawned for invalid entity ID");
}
