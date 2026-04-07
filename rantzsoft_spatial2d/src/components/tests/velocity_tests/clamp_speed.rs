use bevy::prelude::*;

use crate::components::definitions::*;

// ── Velocity2D::clamp() ──────────────────────────────────────

#[test]
fn velocity_clamp_high_speed_clamped_to_max() {
    let v = Velocity2D(Vec2::new(0.0, 800.0));
    let result = v.clamp(200.0, 600.0);
    let speed = result.speed();
    assert!(
        (speed - 600.0).abs() < 1e-3,
        "speed should be clamped to max 600.0, got {speed}"
    );
}

#[test]
fn velocity_clamp_low_speed_clamped_to_min() {
    let v = Velocity2D(Vec2::new(0.0, 100.0));
    let result = v.clamp(200.0, 600.0);
    let speed = result.speed();
    assert!(
        (speed - 200.0).abs() < 1e-3,
        "speed should be clamped to min 200.0, got {speed}"
    );
}

#[test]
fn velocity_clamp_within_bounds_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity within bounds should be unchanged");
}

#[test]
fn velocity_clamp_zero_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "zero velocity should stay zero");
}

#[test]
fn velocity_clamp_preserves_direction() {
    let v = Velocity2D(Vec2::new(300.0, 400.0)); // speed 500, direction (0.6, 0.8)
    let result = v.clamp(200.0, 400.0);
    let dir = result.0.normalize();
    assert!(
        (dir.x - 0.6).abs() < 1e-5,
        "direction x should be 0.6, got {}",
        dir.x
    );
    assert!(
        (dir.y - 0.8).abs() < 1e-5,
        "direction y should be 0.8, got {}",
        dir.y
    );
}

#[test]
fn velocity_clamp_exactly_at_min_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 200.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity exactly at min should be unchanged");
}

#[test]
fn velocity_clamp_exactly_at_max_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 600.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity exactly at max should be unchanged");
}

#[test]
fn velocity_clamp_preserves_negative_direction() {
    let v = Velocity2D(Vec2::new(-300.0, -400.0)); // speed 500
    let result = v.clamp(200.0, 400.0);
    assert!(
        (result.speed() - 400.0).abs() < 1e-3,
        "speed should be clamped to 400.0, got {}",
        result.speed()
    );
    let dir = result.0.normalize();
    assert!(
        (dir.x - (-0.6)).abs() < 1e-5,
        "direction x should be -0.6, got {}",
        dir.x
    );
    assert!(
        (dir.y - (-0.8)).abs() < 1e-5,
        "direction y should be -0.8, got {}",
        dir.y
    );
}
