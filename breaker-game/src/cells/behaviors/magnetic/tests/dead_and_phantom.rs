//! Part D: Dead cell and `PhantomPhase` filtering tests (behaviors 14-17b).

use std::time::Duration;

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::behaviors::phantom::components::PhantomPhase, prelude::*};

// ── Behavior 14: Dead magnetic cell is ignored ──

#[test]
fn dead_magnetic_cell_does_not_apply_force() {
    let mut app = build_magnetic_test_app();

    let magnet = spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    // Insert Dead component
    app.world_mut().entity_mut(magnet).insert(Dead);

    let bolt = spawn_test_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "dead magnetic cell should not apply force, got vx={}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "dead magnetic cell should not affect y velocity, got vy={}",
        vel.0.y
    );
}

// ── Behavior 15: PhantomPhase::Ghost suppresses force ──

#[test]
fn phantom_ghost_magnetic_cell_does_not_apply_force() {
    let mut app = build_magnetic_test_app();

    let magnet = spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    app.world_mut()
        .entity_mut(magnet)
        .insert(PhantomPhase::Ghost);

    let bolt = spawn_test_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "Ghost phase magnetic cell should not apply force, got vx={}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "Ghost phase magnetic cell should not affect y velocity, got vy={}",
        vel.0.y
    );
}

#[test]
fn phantom_telegraph_magnetic_cell_does_apply_force() {
    let mut app = build_magnetic_test_app();

    let magnet = spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    app.world_mut()
        .entity_mut(magnet)
        .insert(PhantomPhase::Telegraph);

    let bolt = spawn_test_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Telegraph phase should NOT suppress force (only Ghost suppresses)
    assert!(
        vel.0.x < 0.0,
        "Telegraph phase magnetic cell should apply force (pull toward origin), got vx={}",
        vel.0.x
    );
}

// ── Behavior 16: PhantomPhase::Solid applies force normally ──

#[test]
fn phantom_solid_magnetic_cell_applies_force() {
    let mut app = build_magnetic_test_app();

    let magnet = spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    app.world_mut()
        .entity_mut(magnet)
        .insert(PhantomPhase::Solid);

    let bolt = spawn_test_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.x < 0.0,
        "Solid phase magnetic cell should apply force, got vx={}",
        vel.0.x
    );
}

// ── Behavior 17: Non-phantom magnetic cell (no PhantomPhase component) applies force ──

#[test]
fn non_phantom_magnetic_cell_applies_force() {
    let mut app = build_magnetic_test_app();

    // Spawn without any PhantomPhase component (Option is None)
    spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);

    let bolt = spawn_test_bolt(&mut app, Vec2::new(50.0, 0.0), Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.x < 0.0,
        "non-phantom magnetic cell (no PhantomPhase) should apply force, got vx={}",
        vel.0.x
    );
}

// ── Behavior 17b: Zero-velocity bolt gains velocity without NaN ──

#[test]
fn zero_velocity_bolt_gains_velocity_from_magnetic_pull() {
    let mut app = build_magnetic_test_app();

    spawn_magnetic_cell(&mut app, Vec2::new(50.0, 0.0), 200.0, 1000.0, 5.0);
    let bolt = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::ZERO, 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        !vel.0.x.is_nan() && !vel.0.y.is_nan(),
        "zero-velocity bolt should not produce NaN, got {:?}",
        vel.0
    );
    assert!(
        vel.0.x > 0.0,
        "zero-velocity bolt should gain positive x velocity (pulled toward magnet at x=50), got {}",
        vel.0.x
    );
    assert!(
        vel.0.y.abs() < f32::EPSILON,
        "zero-velocity bolt should have no y change, got {}",
        vel.0.y
    );
}
