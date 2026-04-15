//! Tests for bolt spawning on subsequent nodes (behaviors 22-23).

use bevy::prelude::*;

use super::helpers::test_app;
use crate::{prelude::*, state::run::NodeOutcome};

// ── Behavior 22: Non-zero velocity on subsequent nodes ────────────────

#[test]
fn spawned_bolt_has_nonzero_velocity_subsequent_node() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
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
        vel.0.length() > f32::EPSILON,
        "bolt should have non-zero velocity on subsequent node, got {:?}",
        vel.0
    );
    assert!(
        vel.0.y > 0.0,
        "bolt velocity should be upward-ish (y > 0), got {:?}",
        vel.0
    );
}

#[test]
fn spawned_bolt_has_nonzero_velocity_node_index_5() {
    // Edge case: node_index: 5
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 5;
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
        vel.0.length() > f32::EPSILON,
        "bolt should have non-zero velocity on node_index 5"
    );
}

#[test]
fn spawned_bolt_velocity_magnitude_matches_base_speed_subsequent_node() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
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
    let speed = vel.0.length();
    // base_speed from BoltDefinition: 720.0
    assert!(
        (speed - 720.0).abs() < 1.0,
        "bolt speed magnitude should be approximately 720.0, got {speed}",
    );
}

// ── Behavior 23: No BoltServing on subsequent nodes ───────────────────

#[test]
fn spawned_bolt_does_not_have_serving_subsequent_node() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");
    assert!(
        app.world().get::<BoltServing>(entity).is_none(),
        "bolt should NOT have BoltServing on subsequent node (node_index > 0)"
    );
}
