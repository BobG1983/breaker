//! Tests for breaker spawning behaviors (2-11).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Scale2D};

use super::helpers::{make_aegis_breaker_definition, test_app};
use crate::{
    breaker::{
        components::{
            Breaker, BreakerInitialized, BreakerReflectionSpread, ExtraBreaker, PrimaryBreaker,
        },
        definition::BreakerDefinition,
        messages::BreakerSpawned,
        registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    effect::effects::life_lost::LivesCount,
    shared::{CleanupOnNodeExit, CleanupOnRunEnd},
};

// ── Behavior 2: Spawns exactly one breaker entity when none exists ────

#[test]
fn setup_run_spawns_exactly_one_breaker() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "setup_run should create exactly 1 breaker entity");
}

#[test]
fn setup_run_spawns_only_matching_breaker_from_registry() {
    let mut app = test_app();
    // Add a second definition to the registry
    let chrono_def: BreakerDefinition =
        ron::de::from_str(r#"(name: "Chrono", life_pool: None, effects: [])"#)
            .expect("test RON should parse");
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Chrono".to_string(), chrono_def);

    app.update();

    // Should still spawn only the one matching SelectedBreaker("Aegis")
    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "should spawn exactly 1 breaker even with multiple in registry"
    );
}

// ── Behavior 3: Spawned breaker has PrimaryBreaker marker ─────────────

#[test]
fn spawned_breaker_has_primary_breaker_marker() {
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
fn spawned_breaker_does_not_have_extra_breaker_marker() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<ExtraBreaker>(entity).is_none(),
        "spawned breaker should NOT have ExtraBreaker"
    );
}

// ── Behavior 4: Spawned breaker has CleanupOnRunEnd marker ────────────

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
fn spawned_breaker_does_not_have_cleanup_on_node_exit() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<CleanupOnNodeExit>(entity).is_none(),
        "primary breaker should NOT have CleanupOnNodeExit (persists across nodes)"
    );
}

// ── Behavior 5: Spawned breaker has BreakerInitialized marker ─────────

#[test]
fn spawned_breaker_has_breaker_initialized() {
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

// ── Behavior 6: Spawned breaker has Position2D at definition y_position

#[test]
fn spawned_breaker_has_position2d_at_default_y() {
    // BreakerDefinition default y_position is -250.0
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
fn spawned_breaker_has_position2d_at_custom_y() {
    // Edge case: non-default y_position of -200.0
    let mut app = test_app();
    let mut custom_def = make_aegis_breaker_definition();
    custom_def.y_position = -200.0;
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Aegis".to_string(), custom_def);

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
    let expected = Vec2::new(0.0, -200.0);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "breaker Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

// ── Behavior 7: Spawned breaker has Scale2D matching definition ───────

#[test]
fn spawned_breaker_has_scale2d_from_definition() {
    // BreakerDefinition default: width=120.0, height=20.0
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

// ── Behavior 8: Spawned breaker has MaxSpeed from definition ──────────

#[test]
fn spawned_breaker_has_max_speed_from_definition() {
    // BreakerDefinition default: max_speed=500.0
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

// ── Behavior 9: Spawned breaker has LivesCount from definition ────────

#[test]
fn spawned_breaker_has_lives_count_some_3() {
    // Aegis definition has life_pool: Some(3)
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
        "LivesCount should be Some(3) for Aegis, got {:?}",
        lives.0
    );
}

#[test]
fn spawned_breaker_has_lives_count_none_for_chrono() {
    // Edge case: Chrono has life_pool: None
    let mut app = test_app();
    let chrono_def: BreakerDefinition =
        ron::de::from_str(r#"(name: "Chrono", life_pool: None, effects: [])"#)
            .expect("test RON should parse");
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Chrono".to_string(), chrono_def);
    app.insert_resource(SelectedBreaker("Chrono".to_string()));

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
        lives.0, None,
        "LivesCount should be None for Chrono, got {:?}",
        lives.0
    );
}

// ── Behavior 10: Spawned breaker has BreakerReflectionSpread in radians

#[test]
fn spawned_breaker_has_reflection_spread_in_radians() {
    // BreakerDefinition default: reflection_spread=75.0 degrees
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

// ── Behavior 11: setup_run sends BreakerSpawned message ───────────────

#[test]
fn setup_run_sends_breaker_spawned_message() {
    let mut app = test_app();
    app.update();

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "setup_run must send BreakerSpawned message"
    );
}
