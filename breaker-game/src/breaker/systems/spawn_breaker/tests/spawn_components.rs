//! Tests for breaker entity creation and its spatial/transform components,
//! messages, and double-spawn prevention.

use bevy::prelude::*;
use rantzsoft_spatial2d::{
    components::{
        InterpolateTransform2D, Position2D, PreviousPosition, Rotation2D, Scale2D, Spatial2D,
    },
    draw_layer::DrawLayer,
};

use super::{super::system::*, helpers::*};
use crate::{
    breaker::{components::Breaker, messages::BreakerSpawned, resources::BreakerConfig},
    shared::GameDrawLayer,
};

#[test]
fn spawn_breaker_creates_entity() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn spawned_breaker_has_spatial2d_components() {
    // Given: no breaker exists
    // When: spawn_breaker runs
    // Then: breaker has Spatial2D, InterpolateTransform2D, GameDrawLayer::Breaker,
    //       Position2D, PreviousPosition, Scale2D, Rotation2D, Transform::default()
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    let world = app.world();
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "breaker should have Spatial2D marker"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "breaker should have InterpolateTransform2D marker"
    );
    assert!(
        world.get::<Position2D>(entity).is_some(),
        "breaker should have Position2D"
    );
    assert!(
        world.get::<PreviousPosition>(entity).is_some(),
        "breaker should have PreviousPosition"
    );
    assert!(
        world.get::<Rotation2D>(entity).is_some(),
        "breaker should have Rotation2D"
    );
    assert!(
        world.get::<Scale2D>(entity).is_some(),
        "breaker should have Scale2D"
    );
    let layer = world
        .get::<GameDrawLayer>(entity)
        .expect("breaker should have GameDrawLayer");
    assert!(
        layer.z().abs() < f32::EPSILON,
        "GameDrawLayer::Breaker.z() should be 0.0, got {}",
        layer.z(),
    );
}

#[test]
fn spawned_breaker_has_position2d_at_spawn_position() {
    // Given: BreakerConfig default y_position=-250.0
    // When: spawn_breaker runs
    // Then: Position2D(Vec2::new(0.0, -250.0))
    let mut app = test_app();
    app.update();

    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("breaker should have Position2D");
    let expected = Vec2::new(0.0, config.y_position);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "breaker Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn spawned_breaker_previous_position_matches_initial_position() {
    // Edge case: PreviousPosition.0 must match Position2D.0 to prevent
    // interpolation teleport on the first frame
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let pos = app
        .world()
        .get::<Position2D>(entity)
        .expect("breaker should have Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(entity)
        .expect("breaker should have PreviousPosition");
    assert_eq!(
        pos.0, prev.0,
        "PreviousPosition should match initial Position2D to prevent teleport"
    );
}

#[test]
fn spawned_breaker_has_scale2d_matching_dimensions() {
    // Given: BreakerConfig default width=120.0, height=20.0
    // When: spawn_breaker runs
    // Then: Scale2D { x: 120.0, y: 20.0 }
    let mut app = test_app();
    app.update();

    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("breaker should have Scale2D");
    assert!(
        (scale.x - config.width).abs() < f32::EPSILON
            && (scale.y - config.height).abs() < f32::EPSILON,
        "Scale2D should be ({}, {}), got ({}, {})",
        config.width,
        config.height,
        scale.x,
        scale.y,
    );
}

#[test]
fn spawned_breaker_has_default_transform() {
    // After migration, Transform should be default (propagation handles it)
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let transform = app
        .world()
        .get::<Transform>(entity)
        .expect("breaker should have Transform");
    assert_eq!(
        *transform,
        Transform::default(),
        "breaker Transform should be default after spatial2d migration, got {transform:?}"
    );
}

#[test]
fn spawn_breaker_sends_breaker_spawned_message() {
    let mut app = test_app();
    app.update();

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_breaker must send BreakerSpawned message"
    );
}

#[test]
fn no_double_spawn() {
    let mut app = test_app();
    app.update();

    // Run spawn_breaker again (simulating a second node entry)
    app.add_systems(Update, spawn_breaker);
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "should not double-spawn breaker");
}

#[test]
fn existing_breaker_still_sends_breaker_spawned() {
    let mut app = test_app();
    app.update(); // First spawn

    // Run spawn_breaker again -- breaker already exists
    app.add_systems(Update, spawn_breaker);
    app.update();

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_breaker must send BreakerSpawned even when breaker already exists"
    );
}
