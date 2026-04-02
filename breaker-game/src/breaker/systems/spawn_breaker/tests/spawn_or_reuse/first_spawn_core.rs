//! Tests for the `spawn_or_reuse_breaker` system — first spawn, core identity.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Scale2D};

use super::super::helpers::*;
use crate::{
    breaker::{
        components::{Breaker, BreakerInitialized, BreakerReflectionSpread, PrimaryBreaker},
        messages::BreakerSpawned,
    },
    effect::effects::life_lost::LivesCount,
    shared::CleanupOnRunEnd,
};

// ── Behavior 1: First node spawns a fully-initialized breaker ──────────

#[test]
fn first_node_spawns_breaker_entity() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "spawn_or_reuse_breaker should create exactly 1 breaker entity"
    );
}

#[test]
fn spawned_breaker_has_primary_marker() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<PrimaryBreaker>(entity).is_some(),
        "spawned breaker should have PrimaryBreaker marker"
    );
}

#[test]
fn spawned_breaker_has_initialized_marker() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<BreakerInitialized>(entity).is_some(),
        "spawned breaker should have BreakerInitialized marker"
    );
}

#[test]
fn spawned_breaker_has_cleanup_on_run_end() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<CleanupOnRunEnd>(entity).is_some(),
        "spawned breaker should have CleanupOnRunEnd"
    );
}

#[test]
fn spawned_breaker_has_lives_count_from_definition() {
    // Given: Aegis definition has life_pool: Some(3)
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("breaker should have LivesCount");
    assert_eq!(
        lives.0,
        Some(3),
        "LivesCount should be Some(3) from Aegis definition"
    );
}

#[test]
fn spawned_breaker_has_position2d_at_definition_y() {
    // Given: BreakerDefinition default y_position=-250.0
    let mut app = test_app();
    app.update();

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
    let expected = Vec2::new(0.0, -250.0);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "breaker Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn spawned_breaker_has_scale2d_matching_definition() {
    // Given: BreakerDefinition default width=120.0, height=20.0
    let mut app = test_app();
    app.update();

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
        (scale.x - 120.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D should be (120.0, 20.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn spawned_breaker_has_max_speed_from_definition() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let max_speed = app
        .world()
        .get::<MaxSpeed>(entity)
        .expect("breaker should have MaxSpeed");
    assert!(
        (max_speed.0 - 500.0).abs() < f32::EPSILON,
        "MaxSpeed should be 500.0, got {}",
        max_speed.0
    );
}

#[test]
fn spawned_breaker_has_reflection_spread_in_radians() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let spread = app
        .world()
        .get::<BreakerReflectionSpread>(entity)
        .expect("breaker should have BreakerReflectionSpread");
    let expected = 75.0_f32.to_radians();
    assert!(
        (spread.0 - expected).abs() < 1e-5,
        "BreakerReflectionSpread should be {expected} (75 degrees in radians), got {}",
        spread.0
    );
}

#[test]
fn spawned_breaker_sends_breaker_spawned_message() {
    let mut app = test_app();
    app.update();

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_or_reuse_breaker must send BreakerSpawned message"
    );
}
