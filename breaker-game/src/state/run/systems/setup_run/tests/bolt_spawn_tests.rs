//! Tests for bolt spawning behaviors on first node (12-21).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinSpeed, Position2D, Scale2D, Velocity2D,
};
use rantzsoft_stateflow::CleanupOnExit;

use super::helpers::{make_aegis_breaker_definition, test_app};
use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltRadius, BoltServing, ExtraBolt, PrimaryBolt},
        definition::BoltDefinition,
        messages::BoltSpawned,
        registry::BoltRegistry,
    },
    breaker::registry::BreakerRegistry,
    state::types::{NodeState, RunState},
};

// ── Behavior 12: Spawns exactly one bolt entity (first node) ──────────

#[test]
fn setup_run_spawns_exactly_one_bolt_first_node() {
    let mut app = test_app();
    // NodeOutcome defaults to node_index: 0
    app.update();

    let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(count, 1, "setup_run should create exactly 1 bolt entity");
}

#[test]
fn setup_run_spawns_only_matching_bolt_from_registry() {
    let mut app = test_app();
    // Add a second bolt definition
    let heavy_def = BoltDefinition {
        name: "Heavy".to_string(),
        base_speed: 500.0,
        min_speed: 250.0,
        max_speed: 1000.0,
        radius: 20.0,
        base_damage: 25.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    app.world_mut()
        .resource_mut::<BoltRegistry>()
        .insert("Heavy".to_string(), heavy_def);

    app.update();

    // Should spawn exactly 1 bolt matching Aegis definition's bolt field ("Bolt")
    let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        count, 1,
        "should spawn exactly 1 bolt even with multiple in registry"
    );
}

// ── Behavior 13: Spawned bolt has PrimaryBolt marker ──────────────────

#[test]
fn spawned_bolt_has_primary_bolt_marker() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world().get::<PrimaryBolt>(entity).is_some(),
        "spawned bolt should have PrimaryBolt marker"
    );
}

#[test]
fn spawned_bolt_does_not_have_extra_bolt_marker() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world().get::<ExtraBolt>(entity).is_none(),
        "spawned bolt should NOT have ExtraBolt"
    );
}

// ── Behavior 14: Spawned bolt has CleanupOnExit<RunState> marker ──────────────

#[test]
fn spawned_bolt_has_cleanup_on_run_end() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world().get::<CleanupOnExit<RunState>>(entity).is_some(),
        "primary bolt should have CleanupOnExit<RunState>"
    );
}

#[test]
fn spawned_bolt_does_not_have_cleanup_on_node_exit() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_none(),
        "primary bolt should NOT have CleanupOnExit<NodeState>"
    );
}

// ── Behavior 15: Spawned bolt has BoltServing marker on first node ────

#[test]
fn spawned_bolt_has_serving_marker_first_node() {
    let mut app = test_app();
    // NodeOutcome defaults to node_index: 0
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world().get::<BoltServing>(entity).is_some(),
        "bolt should have BoltServing on first node (node_index == 0)"
    );
}

// ── Behavior 16: Spawned bolt has zero velocity on first node ─────────

#[test]
fn spawned_bolt_has_zero_velocity_first_node() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let vel = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("bolt should have Velocity2D");
    assert!(
        vel.0.length() < f32::EPSILON,
        "serving bolt should have zero velocity, got {:?}",
        vel.0
    );
}

// ── Behavior 17: Spawned bolt position is above breaker y_position ────

#[test]
fn spawned_bolt_position_above_breaker_default_y() {
    // BreakerDefinition default y_position: -250.0
    // DEFAULT_BOLT_SPAWN_OFFSET_Y: 54.0
    // Expected bolt position: (0.0, -196.0)
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("bolt should have Position2D");
    let expected = Vec2::new(0.0, -196.0);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?} (-250.0 + 54.0), got {:?}",
        position.0,
    );
}

#[test]
fn spawned_bolt_position_above_breaker_custom_y() {
    // Edge case: BreakerDefinition with y_position: -200.0
    // Expected bolt position: (0.0, -146.0) = -200.0 + 54.0
    let mut app = test_app();
    let mut custom_def = make_aegis_breaker_definition();
    custom_def.y_position = -200.0;
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Aegis".to_string(), custom_def);

    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("bolt should have Position2D");
    let expected = Vec2::new(0.0, -146.0);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?} (-200.0 + 54.0), got {:?}",
        position.0,
    );
}

// ── Behavior 18: Spawned bolt has speed components from BoltDefinition

#[test]
fn spawned_bolt_has_speed_components() {
    // BoltDefinition: base_speed=720.0, min_speed=360.0, max_speed=1440.0
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let base = app
        .world()
        .get::<BaseSpeed>(entity)
        .expect("bolt should have BaseSpeed");
    assert!(
        (base.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0, got {}",
        base.0
    );

    let min = app
        .world()
        .get::<MinSpeed>(entity)
        .expect("bolt should have MinSpeed");
    assert!(
        (min.0 - 360.0).abs() < f32::EPSILON,
        "MinSpeed should be 360.0, got {}",
        min.0
    );

    let max = app
        .world()
        .get::<MaxSpeed>(entity)
        .expect("bolt should have MaxSpeed");
    assert!(
        (max.0 - 1440.0).abs() < f32::EPSILON,
        "MaxSpeed should be 1440.0, got {}",
        max.0
    );
}

// ── Behavior 19: Spawned bolt has BoltRadius and radius-based Scale2D ─

#[test]
fn spawned_bolt_has_radius_and_scale() {
    // BoltDefinition: radius=14.0
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let radius = app
        .world()
        .get::<BoltRadius>(entity)
        .expect("bolt should have BoltRadius");
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0, got {}",
        radius.0
    );

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON && (scale.y - 14.0).abs() < f32::EPSILON,
        "Scale2D should be (14.0, 14.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 20: Spawned bolt has BoltBaseDamage ──────────────────────

#[test]
fn spawned_bolt_has_base_damage() {
    // BoltDefinition: base_damage=10.0
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let damage = app
        .world()
        .get::<BoltBaseDamage>(entity)
        .expect("bolt should have BoltBaseDamage");
    assert!(
        (damage.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        damage.0
    );
}

// ── Behavior 21: setup_run sends BoltSpawned message ──────────────────

#[test]
fn setup_run_sends_bolt_spawned_message() {
    let mut app = test_app();
    app.update();

    let messages = app.world().resource::<Messages<BoltSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "setup_run must send BoltSpawned message"
    );
}
