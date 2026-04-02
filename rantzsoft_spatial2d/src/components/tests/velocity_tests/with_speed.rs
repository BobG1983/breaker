use bevy::prelude::*;

use super::super::super::definitions::*;

// ── Velocity2D::with_speed() ────────────────────────────────

#[test]
fn velocity_with_speed_sets_magnitude() {
    let v = Velocity2D(Vec2::new(3.0, 4.0)); // speed 5
    let result = v.with_speed(10.0);
    assert!(
        (result.speed() - 10.0).abs() < 1e-3,
        "speed should be 10.0, got {}",
        result.speed()
    );
}

#[test]
fn velocity_with_speed_preserves_direction() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.with_speed(10.0);
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
fn velocity_with_speed_zero_velocity_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.with_speed(500.0);
    assert_eq!(result, v, "zero velocity with_speed should stay zero");
}

#[test]
fn velocity_with_speed_preserves_negative_direction() {
    let v = Velocity2D(Vec2::new(-3.0, -4.0)); // speed 5, direction (-0.6, -0.8)
    let result = v.with_speed(10.0);
    assert!(
        (result.speed() - 10.0).abs() < 1e-3,
        "speed should be 10.0, got {}",
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

#[test]
fn velocity_with_speed_zero_produces_zero_vector() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.with_speed(0.0);
    assert!(
        result.speed() < f32::EPSILON,
        "with_speed(0) should produce zero magnitude, got {}",
        result.speed()
    );
}
