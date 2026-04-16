//! Part B: Single magnet integration tests (behaviors 7-9).

use std::time::Duration;

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 7: Single magnet pulls bolt toward its center ──

#[test]
fn single_magnet_pulls_bolt_toward_center() {
    let mut app = build_magnetic_test_app();

    // Magnetic cell at origin, radius 200, strength 1000, half_width 5
    spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    // Bolt at (100, 0), velocity (0, 400), base speed 400
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(100.0, 0.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Bolt should be pulled toward the magnet at origin (negative x direction)
    assert!(
        vel.0.x < 0.0,
        "bolt velocity x should be negative (pulled toward magnet at origin), got {}",
        vel.0.x
    );
    // y should still be positive (original upward motion mostly preserved)
    assert!(
        vel.0.y > 0.0,
        "bolt velocity y should still be positive, got {}",
        vel.0.y
    );

    // Verify approximate velocity delta: -1000.0 / 10000.0 * (1/60) = -0.001667
    let expected_dx = -1000.0 / 10000.0 * (1.0 / 60.0);
    assert!(
        (vel.0.x - expected_dx).abs() < 0.001,
        "velocity x delta should be approximately {}, got {}",
        expected_dx,
        vel.0.x
    );
}

#[test]
fn bolt_at_radius_boundary_still_affected() {
    let mut app = build_magnetic_test_app();

    // Magnetic cell at origin, radius 200
    spawn_magnetic_cell(&mut app, Vec2::ZERO, 200.0, 1000.0, 5.0);
    // Bolt exactly at radius boundary (200, 0)
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(200.0, 0.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // At boundary, force should still be applied (distance <= radius)
    assert!(
        vel.0.x < 0.0,
        "bolt at radius boundary should still be pulled toward magnet, got vx={}",
        vel.0.x
    );
}

// ── Behavior 8: Bolt outside magnet radius is unaffected ──

#[test]
fn bolt_outside_radius_is_unaffected() {
    let mut app = build_magnetic_test_app();

    // Magnetic cell at origin, radius 100
    spawn_magnetic_cell(&mut app, Vec2::ZERO, 100.0, 1000.0, 5.0);
    // Bolt at (150, 0) -- outside radius 100
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(150.0, 0.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "bolt outside radius should have unchanged x velocity, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "bolt outside radius should have unchanged y velocity, got {}",
        vel.0.y
    );
}

#[test]
fn bolt_just_outside_radius_is_unaffected() {
    let mut app = build_magnetic_test_app();

    spawn_magnetic_cell(&mut app, Vec2::ZERO, 100.0, 1000.0, 5.0);
    // Bolt at distance 100.001 -- just outside radius
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(100.001, 0.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "bolt just outside radius should have unchanged x velocity, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "bolt just outside radius should have unchanged y velocity, got {}",
        vel.0.y
    );
}

// ── Behavior 9: Bolt at cell center uses min_distance clamping ──

#[test]
fn bolt_at_cell_center_does_not_produce_nan() {
    let mut app = build_magnetic_test_app();

    // Magnetic cell at (50, 50), half_extents.x = 10 = min_distance
    spawn_magnetic_cell(&mut app, Vec2::new(50.0, 50.0), 200.0, 1000.0, 10.0);
    // Bolt exactly at cell center
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(50.0, 50.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        !vel.0.x.is_nan() && !vel.0.y.is_nan(),
        "velocity should not be NaN at coincident position, got {:?}",
        vel.0
    );
    // inverse_square_attraction returns ZERO for coincident positions
    assert!(
        (vel.0.x - 0.0).abs() < f32::EPSILON,
        "velocity x should remain 0.0 at coincident position, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "velocity y should remain 400.0 at coincident position, got {}",
        vel.0.y
    );
}

#[test]
fn bolt_very_close_to_cell_center_uses_min_distance() {
    let mut app = build_magnetic_test_app();

    // Magnetic cell at (50, 50), half_extents.x = 10 = min_distance
    spawn_magnetic_cell(&mut app, Vec2::new(50.0, 50.0), 200.0, 1000.0, 10.0);
    // Bolt very close but not exactly at center
    let bolt = spawn_test_bolt(
        &mut app,
        Vec2::new(50.001, 50.0),
        Vec2::new(0.0, 400.0),
        400.0,
    );

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        !vel.0.x.is_nan() && !vel.0.y.is_nan(),
        "velocity should not be NaN at very close distance, got {:?}",
        vel.0
    );
    // Small force should be applied using min_distance_squared = 100.0
    // The x component should be negative (pulled toward 50.0 from 50.001)
    assert!(
        vel.0.x < 0.0,
        "bolt very close to center should still feel a small force, got vx={}",
        vel.0.x
    );
}
