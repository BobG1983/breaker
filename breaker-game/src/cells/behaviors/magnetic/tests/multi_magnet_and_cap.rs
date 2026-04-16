//! Part C: Multiple magnets and force capping tests (behaviors 10-13).

use std::time::Duration;

use bevy::prelude::*;

use super::helpers::*;
use crate::prelude::*;

// ── Behavior 10: Multiple magnets sum their forces ──

#[test]
fn opposing_magnets_cancel_forces() {
    let mut app = build_magnetic_test_app();

    // Magnet A at (-100, 0), Magnet B at (100, 0), equal strength
    spawn_magnetic_cell(&mut app, Vec2::new(-100.0, 0.0), 300.0, 500.0, 5.0);
    spawn_magnetic_cell(&mut app, Vec2::new(100.0, 0.0), 300.0, 500.0, 5.0);
    // Bolt at origin, equidistant from both
    let bolt = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Forces should cancel -- x change should be negligible
    assert!(
        vel.0.x.abs() < 1e-4,
        "opposing equal magnets should cancel x forces, got vx={}",
        vel.0.x
    );
    // y should remain ~400.0 (no y-component forces from either magnet)
    assert!(
        (vel.0.y - 400.0).abs() < 0.01,
        "y velocity should remain ~400.0, got {}",
        vel.0.y
    );
}

#[test]
fn same_side_magnets_produce_additive_x_force() {
    let mut app = build_magnetic_test_app();

    // Both magnets on the positive x side
    spawn_magnetic_cell(&mut app, Vec2::new(50.0, 0.0), 300.0, 500.0, 5.0);
    spawn_magnetic_cell(&mut app, Vec2::new(100.0, 0.0), 300.0, 500.0, 5.0);
    // Bolt at origin
    let bolt = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0 / 60.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Both magnets pull rightward
    assert!(
        vel.0.x > 0.0,
        "both magnets on same side should produce positive x force, got vx={}",
        vel.0.x
    );
}

// ── Behavior 11: Two magnets on same side produce additive force ──

#[test]
fn two_same_side_magnets_produce_larger_force_than_one() {
    // Test with one magnet
    let mut app_one = build_magnetic_test_app();
    spawn_magnetic_cell(&mut app_one, Vec2::new(50.0, 0.0), 200.0, 500.0, 5.0);
    let bolt_one = spawn_test_bolt(&mut app_one, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);
    advance_to_playing(&mut app_one);
    tick_with_dt(&mut app_one, Duration::from_secs_f32(1.0 / 60.0));
    let vel_one = app_one.world().get::<Velocity2D>(bolt_one).unwrap().0.x;

    // Test with two magnets
    let mut app_two = build_magnetic_test_app();
    spawn_magnetic_cell(&mut app_two, Vec2::new(50.0, 0.0), 200.0, 500.0, 5.0);
    spawn_magnetic_cell(&mut app_two, Vec2::new(80.0, 0.0), 200.0, 500.0, 5.0);
    let bolt_two = spawn_test_bolt(&mut app_two, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);
    advance_to_playing(&mut app_two);
    tick_with_dt(&mut app_two, Duration::from_secs_f32(1.0 / 60.0));
    let vel_two = app_two.world().get::<Velocity2D>(bolt_two).unwrap().0.x;

    assert!(
        vel_two > vel_one,
        "two magnets on same side should produce larger x delta ({vel_two}) than one ({vel_one})"
    );
}

// ── Behavior 12: Total magnetic acceleration is capped at 2 x base_speed ──

#[test]
fn acceleration_capped_at_two_times_base_speed() {
    let mut app = build_magnetic_test_app();

    // Absurdly strong magnet very close
    spawn_magnetic_cell(&mut app, Vec2::new(5.0, 0.0), 200.0, 1_000_000.0, 5.0);
    // Bolt at origin, base_speed 400
    let bolt = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    let dt = 1.0_f32 / 60.0;
    tick_with_dt(&mut app, Duration::from_secs_f32(dt));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Uncapped: 1_000_000 / 25 = 40_000
    // Cap: 2 * 400 = 800
    // Max velocity delta = 800 * dt = 800 * (1/60) = 13.333...
    let max_delta = 2.0 * 400.0 * dt;
    let actual_delta_x = vel.0.x; // started at 0.0 in x
    assert!(
        actual_delta_x.abs() <= max_delta + 0.01,
        "velocity delta magnitude should be at most {}, got {}",
        max_delta,
        actual_delta_x.abs()
    );
    // Verify the velocity isn't enormous (would be ~666 without cap)
    assert!(
        vel.0.x.abs() < 20.0,
        "velocity x should be capped to a small value, not enormous, got {}",
        vel.0.x
    );
}

#[test]
fn acceleration_exactly_at_cap_passes_through() {
    let mut app = build_magnetic_test_app();

    // strength=20000, distance=5, accel = 20000/25 = 800 = cap (2*400)
    // Cap uses > not >=, so exactly at cap should pass through unchanged
    spawn_magnetic_cell(&mut app, Vec2::new(5.0, 0.0), 200.0, 20000.0, 5.0);
    let bolt = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 400.0);

    advance_to_playing(&mut app);
    let dt = 1.0_f32 / 60.0;
    tick_with_dt(&mut app, Duration::from_secs_f32(dt));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // accel = 800.0 exactly, vel delta = 800 * dt = 13.333
    let expected_delta = 800.0 * dt;
    assert!(
        (vel.0.x - expected_delta).abs() < 0.01,
        "acceleration exactly at cap should pass through, expected delta ~{}, got vx={}",
        expected_delta,
        vel.0.x
    );
}

// ── Behavior 13: Acceleration cap uses bolt's own BaseSpeed ──

#[test]
fn acceleration_cap_uses_per_bolt_base_speed() {
    let mut app = build_magnetic_test_app();

    // Very strong magnet to ensure capping triggers for both bolts.
    // Both bolts on the x-axis at the same distance so they get the same
    // uncapped force — the only difference is their BaseSpeed-derived cap.
    spawn_magnetic_cell(&mut app, Vec2::new(5.0, 0.0), 200.0, 1_000_000.0, 5.0);

    // Bolt A: base_speed 200, cap = 2*200 = 400
    let bolt_a = spawn_test_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 200.0), 200.0);
    // Bolt B: base_speed 800, cap = 2*800 = 1600, same position on x-axis
    let bolt_b = spawn_test_bolt(&mut app, Vec2::new(-5.0, 0.0), Vec2::new(0.0, 800.0), 800.0);

    advance_to_playing(&mut app);
    let dt = 1.0_f32 / 60.0;
    tick_with_dt(&mut app, Duration::from_secs_f32(dt));

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();

    // Bolt A max delta = 400 * dt = 6.667
    let max_delta_a = 2.0 * 200.0 * dt;
    // Bolt B max delta = 1600 * dt = 26.667
    let max_delta_b = 2.0 * 800.0 * dt;

    let delta_a = (vel_a.0 - Vec2::new(0.0, 200.0)).length();
    let delta_b = (vel_b.0 - Vec2::new(0.0, 800.0)).length();

    assert!(
        delta_a <= max_delta_a + 0.01,
        "bolt A velocity delta should be capped at {max_delta_a}, got {delta_a}"
    );
    assert!(
        delta_b <= max_delta_b + 0.01,
        "bolt B velocity delta should be capped at {max_delta_b}, got {delta_b}"
    );
    // Bolt B should have a larger velocity delta than bolt A (higher cap)
    assert!(
        delta_b > delta_a,
        "bolt B (higher BaseSpeed) should have larger velocity delta ({delta_b}) than bolt A ({delta_a})"
    );
}
