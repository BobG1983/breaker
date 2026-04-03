use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{super::*, helpers::*};
use crate::{
    bolt::{
        components::{Bolt, BoltServing},
        messages::BoltSpawned,
    },
    state::run::RunState,
};

#[test]
fn first_node_spawns_serving_bolt_with_zero_velocity() {
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let serving_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .count();
    assert_eq!(serving_count, 1, "first node bolt should have BoltServing");

    let velocity = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(
        velocity.speed() < f32::EPSILON,
        "serving bolt should have zero velocity"
    );
}

#[test]
fn subsequent_node_spawns_launched_bolt() {
    let mut app = test_app();
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let serving_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        serving_count, 0,
        "subsequent node bolt should not have BoltServing"
    );

    let velocity = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(velocity.0.y > 0.0, "bolt should launch upward");
}

#[test]
fn spawn_bolt_sends_bolt_spawned_message() {
    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let messages = app.world().resource::<Messages<BoltSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_bolt must send BoltSpawned message"
    );
}

#[test]
fn existing_bolt_is_not_double_spawned() {
    let mut app = test_app();
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .headless()
        .spawn(app.world_mut());
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 1,
        "spawn_bolt should not create a second bolt when one already exists"
    );
}

#[test]
fn existing_bolt_still_triggers_bolt_spawned_message() {
    let mut app = test_app();
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .headless()
        .spawn(app.world_mut());
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let messages = app.world().resource::<Messages<BoltSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_bolt must send BoltSpawned even when bolt already exists"
    );
}

#[test]
fn first_spawn_creates_bolt_with_cleanup_on_run_end() {
    use crate::shared::{CleanupOnNodeExit, CleanupOnRunEnd};

    let mut app = test_app();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    assert!(
        app.world().get::<CleanupOnRunEnd>(entity).is_some(),
        "bolt should have CleanupOnRunEnd marker (persists across nodes)"
    );
    assert!(
        app.world().get::<CleanupOnNodeExit>(entity).is_none(),
        "bolt should NOT have CleanupOnNodeExit (it persists across nodes)"
    );
}
