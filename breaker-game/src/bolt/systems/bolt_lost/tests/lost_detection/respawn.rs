use bevy::prelude::*;
use rantzsoft_spatial2d::components::{PreviousPosition, Spatial2D};

use super::super::helpers::*;
use crate::{
    bolt::{
        components::BoltAngleSpread,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    prelude::*,
    shared::GameDrawLayer,
};

#[test]
fn respawn_inserts_position2d_at_breaker_x() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(200.0, playfield.bottom() - 100.0),
        Vec2::new(100.0, -400.0),
    );
    tick(&mut app);

    let (vel, pos) = app
        .world_mut()
        .query::<(&Velocity2D, &Position2D)>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.0.length();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "respawn speed should equal base_speed 720.0, got {speed:.1}",
    );

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "respawn angle {angle:.3} rad should be within spread {DEFAULT_BOLT_ANGLE_SPREAD:.3} rad",
    );

    assert!(vel.0.y > 0.0, "respawn should launch upward");

    assert!(
        (pos.0.x - breaker_x).abs() < f32::EPSILON,
        "respawn Position2D.0.x should match breaker X {breaker_x:.0}, got {:.1}",
        pos.0.x,
    );
}

#[test]
fn respawn_position2d_y_uses_spawn_offset() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected_y = breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "respawn Position2D.0.y should be breaker_y + spawn_offset_y ({expected_y}), got {}",
        pos.0.y,
    );
}

#[test]
fn respawn_inserts_previous_position_matching_position2d() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let (pos, prev_pos) = app
        .world_mut()
        .query_filtered::<(&Position2D, &PreviousPosition), With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "respawn Position2D should be ({expected:?}), got {:?}",
        pos.0,
    );
    assert!(
        (prev_pos.0 - expected).length() < f32::EPSILON,
        "respawn PreviousPosition should match Position2D ({expected:?}), got {:?}",
        prev_pos.0,
    );
}

// ── Migration tests: Behaviors 33-39 ──

// Behavior 33: bolt_lost queries BoltAngleSpread instead of BoltRespawnAngleSpread
#[test]
fn bolt_lost_queries_bolt_angle_spread_for_respawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    // Verify the entity has BoltAngleSpread (from .definition())
    assert!(
        app.world().get::<BoltAngleSpread>(entity).is_some(),
        "definition-built bolt should have BoltAngleSpread"
    );

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(vel.0.y > 0.0, "bolt should be respawned upward");

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "respawn angle {angle:.3} should be within BoltAngleSpread ({DEFAULT_BOLT_ANGLE_SPREAD:.3})"
    );
}

// Behavior 34: bolt_lost queries BoltSpawnOffsetY instead of BoltRespawnOffsetY
#[test]
fn bolt_lost_queries_bolt_spawn_offset_y_for_respawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(42.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(42.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0.x - expected.x).abs() < f32::EPSILON,
        "respawn x should be {}, got {}",
        expected.x,
        pos.0.x
    );
    assert!(
        (pos.0.y - expected.y).abs() < f32::EPSILON,
        "respawn y should be {} (from BoltSpawnOffsetY {DEFAULT_BOLT_SPAWN_OFFSET_Y}), got {}",
        expected.y,
        pos.0.y
    );
}

// Behavior 37: bolt_lost respawn velocity uses base_speed via velocity formula
#[test]
fn bolt_lost_respawn_velocity_uses_base_speed() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.speed();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "respawn speed should be approximately 720.0 (BaseSpeed from definition), got {speed:.1}"
    );
}

// Behavior 38: bolt_lost respawn inserts PreviousPosition matching new position
#[test]
fn bolt_lost_respawn_previous_position_matches_new_position() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let prev = app.world().get::<PreviousPosition>(entity).unwrap();
    let expected = Vec2::new(0.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "Position2D should be {expected:?}, got {:?}",
        pos.0
    );
    assert!(
        (prev.0 - expected).length() < f32::EPSILON,
        "PreviousPosition should match Position2D at {expected:?}, got {:?}",
        prev.0
    );
}

// ── Birthing: respawned bolt should enter birthing animation ──

#[test]
fn bolt_lost_respawn_inserts_birthing_component() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    // After respawn, bolt should have Birthing component
    assert!(
        app.world().get::<Birthing>(entity).is_some(),
        "respawned bolt must have Birthing component for scale-up animation"
    );
}
