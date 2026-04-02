//! Tests for `fire()`: spawning `ExplodeRequest` at source position,
//! position fallback, `damage` values, and `reverse()` no-op.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::effect::*;

// -- Behavior 19: fire() spawns ExplodeRequest entity at source position ──

#[test]
fn fire_spawns_explode_request_entity_at_source_position() {
    let mut world = World::new();
    let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<(&ExplodeRequest, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one ExplodeRequest entity"
    );

    let (request, pos) = results[0];
    assert!(
        (request.range - 60.0).abs() < f32::EPSILON,
        "expected range 60.0, got {}",
        request.range
    );
    assert!(
        (request.damage - 2.0).abs() < f32::EPSILON,
        "expected damage 2.0, got {}",
        request.damage
    );
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "expected x 50.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 75.0).abs() < f32::EPSILON,
        "expected y 75.0, got {}",
        pos.0.y
    );
}

#[test]
fn fire_with_no_transform_defaults_position_to_zero() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 60.0, 2.0, "", &mut world);

    let mut query = world.query::<(&ExplodeRequest, &Position2D)>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "request should still be spawned");

    let (_, pos) = results[0];
    assert!(
        (pos.0.x).abs() < f32::EPSILON,
        "position should default to 0.0 x"
    );
    assert!(
        (pos.0.y).abs() < f32::EPSILON,
        "position should default to 0.0 y"
    );
}

// -- Behavior 20: fire() with different damage values ──

#[test]
fn fire_with_custom_damage() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 40.0, 1.5, "", &mut world);

    let mut query = world.query::<&ExplodeRequest>();
    let request = query
        .iter(&world)
        .next()
        .expect("ExplodeRequest should exist");
    assert!(
        (request.range - 40.0).abs() < f32::EPSILON,
        "expected range 40.0, got {}",
        request.range
    );
    assert!(
        (request.damage - 1.5).abs() < f32::EPSILON,
        "expected damage 1.5, got {}",
        request.damage
    );
}

#[test]
fn fire_with_zero_damage() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 40.0, 0.0, "", &mut world);

    let mut query = world.query::<&ExplodeRequest>();
    let request = query
        .iter(&world)
        .next()
        .expect("ExplodeRequest should exist");
    assert!(
        (request.damage - 0.0).abs() < f32::EPSILON,
        "expected damage 0.0, got {}",
        request.damage
    );
}

// -- Behavior 26: reverse() is a no-op ──

#[test]
fn reverse_is_noop() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

    // reverse should complete without panicking or modifying anything
    reverse(entity, "", &mut world);

    // Entity still exists
    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}

#[test]
fn reverse_on_empty_entity_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, "", &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "empty entity should still exist after no-op reverse"
    );
}
