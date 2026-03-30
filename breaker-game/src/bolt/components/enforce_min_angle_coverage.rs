use super::definitions::*;
use bevy::prelude::*;

/// Helper: assert speed is preserved within tolerance.
fn assert_speed_preserved(vx: f32, vy: f32, min_deg: f32) {
    let mut velocity = Vec2::new(vx, vy);
    let speed_before = velocity.length();
    if speed_before < f32::EPSILON {
        return;
    }
    enforce_min_angle(&mut velocity, min_deg.to_radians());
    let speed_after = velocity.length();
    assert!(
        (speed_before - speed_after).abs() < 0.1,
        "speed should be preserved: {speed_before} vs {speed_after} (input: ({vx}, {vy}), min_deg: {min_deg})"
    );
}

#[test]
fn preserves_speed_nearly_horizontal_positive() {
    assert_speed_preserved(400.0, 0.01, 15.0);
}

#[test]
fn preserves_speed_nearly_horizontal_negative() {
    assert_speed_preserved(-300.0, -0.01, 30.0);
}

#[test]
fn preserves_speed_diagonal_positive() {
    assert_speed_preserved(200.0, 200.0, 10.0);
}

#[test]
fn preserves_speed_diagonal_negative() {
    assert_speed_preserved(-250.0, -250.0, 45.0);
}

#[test]
fn preserves_speed_nearly_vertical() {
    assert_speed_preserved(0.01, 500.0, 5.0);
}

#[test]
fn preserves_speed_at_boundary_angle() {
    assert_speed_preserved(100.0, 26.8, 15.0); // ~15 deg from horizontal
}

#[test]
fn preserves_speed_large_magnitude() {
    assert_speed_preserved(-499.0, 499.0, 20.0);
}

/// Helper: assert result is finite (no NaN/infinity).
fn assert_finite(vx: f32, vy: f32, min_deg: f32) {
    let mut velocity = Vec2::new(vx, vy);
    enforce_min_angle(&mut velocity, min_deg.to_radians());
    assert!(
        velocity.x.is_finite(),
        "x should be finite: {} (input: ({vx}, {vy}), min_deg: {min_deg})",
        velocity.x
    );
    assert!(
        velocity.y.is_finite(),
        "y should be finite: {} (input: ({vx}, {vy}), min_deg: {min_deg})",
        velocity.y
    );
}

#[test]
fn never_nan_horizontal() {
    assert_finite(500.0, 0.0, 20.0);
}

#[test]
fn never_nan_vertical() {
    assert_finite(0.0, 500.0, 45.0);
}

#[test]
fn never_nan_zero_velocity() {
    assert_finite(0.0, 0.0, 10.0);
}

#[test]
fn never_nan_near_zero() {
    assert_finite(0.001, -0.001, 1.0);
}

#[test]
fn never_nan_large_values() {
    assert_finite(-999.0, 999.0, 89.0);
}

#[test]
fn never_nan_small_min_angle() {
    assert_finite(100.0, -50.0, 1.0);
}

#[test]
fn never_nan_negative_both() {
    assert_finite(-700.0, -700.0, 30.0);
}

/// Helper: assert angle from horizontal meets minimum.
fn assert_meets_minimum(vx: f32, vy: f32, min_deg: f32) {
    let mut velocity = Vec2::new(vx, vy);
    if velocity.length() < f32::EPSILON {
        return;
    }
    let min_rad = min_deg.to_radians();
    enforce_min_angle(&mut velocity, min_rad);
    let angle = velocity.y.abs().atan2(velocity.x.abs());
    assert!(
        angle >= min_rad - 1e-4,
        "angle {angle:.4} should be >= min {min_rad:.4}, vel=({}, {}), input: ({vx}, {vy}), min_deg: {min_deg}",
        velocity.x,
        velocity.y
    );
}

#[test]
fn meets_minimum_shallow_positive() {
    assert_meets_minimum(400.0, 1.0, 15.0);
}

#[test]
fn meets_minimum_shallow_negative() {
    assert_meets_minimum(-400.0, -1.0, 30.0);
}

#[test]
fn meets_minimum_already_steep() {
    assert_meets_minimum(100.0, 400.0, 10.0);
}

#[test]
fn meets_minimum_exactly_at_boundary() {
    // 45 degrees exactly
    assert_meets_minimum(100.0, 100.0, 45.0);
}

#[test]
fn meets_minimum_nearly_horizontal_large_min() {
    assert_meets_minimum(500.0, 0.01, 45.0);
}

#[test]
fn meets_minimum_mixed_signs() {
    assert_meets_minimum(-300.0, 50.0, 20.0);
}

#[test]
fn meets_minimum_small_angle_requirement() {
    assert_meets_minimum(200.0, 10.0, 5.0);
}
