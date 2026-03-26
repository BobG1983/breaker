use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Spatial2D, Velocity2D};

use super::*;
use crate::{
    bolt::{
        components::{Bolt, BoltServing, ExtraBolt},
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    chips::components::{
        BoltSizeBoost, BoltSpeedBoost, ChainHit, DamageBoost, Piercing, PiercingRemaining,
    },
    run::RunState,
    shared::GameDrawLayer,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BoltConfig>()
        .init_resource::<RunState>()
        .add_systems(Update, reset_bolt);
    app
}

/// Spawns a bolt entity with spatial2d components for reset testing.
fn spawn_bolt_entity(app: &mut App, pos: Vec2, velocity: Velocity2D) -> Entity {
    app.world_mut()
        .spawn((Bolt, velocity, Position2D(pos), PreviousPosition(pos)))
        .id()
}

/// Spawns a breaker entity at the given position using `Position2D`.
fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(x, y)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id()
}

#[test]
fn reset_bolt_writes_position2d_above_breaker() {
    // Given: bolt at Position2D(150.0, 100.0), breaker at (0.0, -250.0),
    //        spawn_offset_y = 30.0
    // When: reset_bolt runs
    // Then: Position2D(Vec2::new(0.0, -220.0))
    let mut app = test_app();
    spawn_bolt_entity(
        &mut app,
        Vec2::new(150.0, 100.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let config = BoltConfig::default();
    let expected = Vec2::new(0.0, -250.0 + config.spawn_offset_y);

    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Position2D");

    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn reset_bolt_snaps_previous_position_to_prevent_interpolation_teleport() {
    // Given: bolt with PreviousPosition(140.0, 90.0)
    // When: reset_bolt runs
    // Then: PreviousPosition.0 matches new Position2D.0
    let mut app = test_app();
    let bolt_id = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(300.0, 400.0)),
            Position2D(Vec2::new(150.0, 100.0)),
            PreviousPosition(Vec2::new(140.0, 90.0)),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let pos = app
        .world()
        .get::<Position2D>(bolt_id)
        .expect("bolt should have Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(bolt_id)
        .expect("bolt should have PreviousPosition");
    assert_eq!(
        pos.0, prev.0,
        "PreviousPosition should match Position2D after reset to prevent teleport"
    );
}

#[test]
fn reset_bolt_zeroes_velocity_on_node_zero() {
    let mut app = test_app();
    spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    assert!(
        velocity.0 == Vec2::ZERO,
        "velocity should be zero on node 0, got {:?}",
        velocity.0
    );
}

#[test]
fn reset_bolt_sets_initial_velocity_on_subsequent_nodes() {
    let mut app = test_app();
    app.world_mut().resource_mut::<RunState>().node_index = 2;
    spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(0.0, 0.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let config = BoltConfig::default();
    let velocity = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    assert!(
        velocity.0.y > 0.0,
        "velocity y should be positive on subsequent node, got {}",
        velocity.0.y
    );
    let speed = velocity.speed();
    assert!(
        (speed - config.base_speed).abs() < 1.0,
        "speed should be approximately base_speed ({:.1}), got {speed:.1}",
        config.base_speed
    );
}

#[test]
fn reset_bolt_inserts_serving_on_node_zero() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(0.0, 0.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_some(),
        "bolt should have BoltServing on node 0"
    );
}

#[test]
fn reset_bolt_removes_serving_on_subsequent_nodes() {
    let mut app = test_app();
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    let bolt_id = app
        .world_mut()
        .spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            PreviousPosition(Vec2::new(0.0, 0.0)),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_none(),
        "bolt should NOT have BoltServing on node 1"
    );
}

#[test]
fn reset_bolt_resets_piercing_remaining_to_piercing() {
    let mut app = test_app();
    let bolt_id = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            PreviousPosition(Vec2::new(0.0, 0.0)),
            Piercing(3),
            PiercingRemaining(0),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let remaining = app
        .world()
        .get::<PiercingRemaining>(bolt_id)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        remaining.0, 3,
        "PiercingRemaining should be reset to Piercing(3), got {}",
        remaining.0
    );
}

#[test]
fn reset_bolt_does_not_touch_chip_effect_components() {
    let mut app = test_app();
    let bolt_id = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            PreviousPosition(Vec2::new(0.0, 0.0)),
            Piercing(3),
            DamageBoost(0.5),
            BoltSpeedBoost(100.0),
            BoltSizeBoost(2.0),
            ChainHit(1),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let world = app.world();
    assert_eq!(
        world.get::<Piercing>(bolt_id),
        Some(&Piercing(3)),
        "Piercing should be unchanged"
    );
    assert_eq!(
        world.get::<DamageBoost>(bolt_id),
        Some(&DamageBoost(0.5)),
        "DamageBoost should be unchanged"
    );
    assert_eq!(
        world.get::<BoltSpeedBoost>(bolt_id),
        Some(&BoltSpeedBoost(100.0)),
        "BoltSpeedBoost should be unchanged"
    );
    assert_eq!(
        world.get::<BoltSizeBoost>(bolt_id),
        Some(&BoltSizeBoost(2.0)),
        "BoltSizeBoost should be unchanged"
    );
    assert_eq!(
        world.get::<ChainHit>(bolt_id),
        Some(&ChainHit(1)),
        "ChainHit should be unchanged"
    );
}

#[test]
fn reset_bolt_is_noop_when_no_bolt_exists() {
    let mut app = test_app();
    spawn_breaker(&mut app, 0.0, -250.0);

    // Should not panic
    app.update();

    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(bolt_count, 0, "no bolt should be created by reset");
}

#[test]
fn reset_bolt_ignores_extra_bolt_entities() {
    let mut app = test_app();
    // Baseline bolt
    let baseline_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(150.0, 100.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );

    // Extra bolt at a different position
    let extra_id = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(200.0, 300.0)),
            Position2D(Vec2::new(-100.0, 50.0)),
            PreviousPosition(Vec2::new(-100.0, 50.0)),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let config = BoltConfig::default();
    let expected_y = -250.0 + config.spawn_offset_y;

    // Baseline should be repositioned
    let baseline_pos = app.world().get::<Position2D>(baseline_id).unwrap();
    assert!(
        (baseline_pos.0.y - expected_y).abs() < f32::EPSILON,
        "baseline bolt should be repositioned to y={expected_y}, got y={}",
        baseline_pos.0.y,
    );

    // Extra bolt should remain untouched
    let extra_pos = app.world().get::<Position2D>(extra_id).unwrap();
    assert!(
        (extra_pos.0.x - (-100.0)).abs() < f32::EPSILON,
        "extra bolt x should be unchanged at -100.0, got {}",
        extra_pos.0.x,
    );
    assert!(
        (extra_pos.0.y - 50.0).abs() < f32::EPSILON,
        "extra bolt y should be unchanged at 50.0, got {}",
        extra_pos.0.y,
    );
}
