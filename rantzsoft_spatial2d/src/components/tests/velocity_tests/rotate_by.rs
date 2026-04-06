use bevy::prelude::*;

use crate::components::definitions::*;

// ── Velocity2D::rotate_by() ────────────────────────────────

#[test]
fn rotate_by_zero_is_unchanged() {
    let v = Velocity2D(Vec2::new(100.0, 300.0));
    let result = v.rotate_by(0.0);
    assert!((result.0.x - v.0.x).abs() < 1e-3, "x should be unchanged");
    assert!((result.0.y - v.0.y).abs() < 1e-3, "y should be unchanged");
}

#[test]
fn rotate_by_preserves_speed() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.rotate_by(0.5);
    assert!(
        (result.speed() - 5.0).abs() < 1e-3,
        "speed should be preserved at 5.0, got {}",
        result.speed()
    );
}

#[test]
fn rotate_by_positive_rotates_clockwise() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(std::f32::consts::FRAC_PI_2);
    assert!(
        (result.0.x - 400.0).abs() < 1e-3,
        "x should be 400, got {}",
        result.0.x
    );
    assert!(
        result.0.y.abs() < 1e-3,
        "y should be ~0, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_negative_rotates_counterclockwise() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(-std::f32::consts::FRAC_PI_2);
    assert!(
        (result.0.x - (-400.0)).abs() < 1e-3,
        "x should be -400, got {}",
        result.0.x
    );
    assert!(
        result.0.y.abs() < 1e-3,
        "y should be ~0, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_small_angle_from_up_tilts_right() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(2.0_f32.to_radians());
    assert!(
        result.0.x > 0.0,
        "small positive rotation should tilt x positive"
    );
    assert!(
        result.0.y > 0.0,
        "small rotation from up should keep y positive"
    );
}

#[test]
fn rotate_by_small_negative_from_up_tilts_left() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(-2.0_f32.to_radians());
    assert!(
        result.0.x < 0.0,
        "small negative rotation should tilt x negative"
    );
    assert!(
        result.0.y > 0.0,
        "small rotation from up should keep y positive"
    );
}

#[test]
fn rotate_by_pi_reverses_direction() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(std::f32::consts::PI);
    assert!(
        result.0.x.abs() < 1e-3,
        "x should be ~0, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - (-400.0)).abs() < 1e-3,
        "y should be -400, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_from_diagonal_stays_correct() {
    // Start at 45 degrees (upper-right), rotate 90 clockwise → lower-right
    let v = Velocity2D(Vec2::new(283.0, 283.0)); // ~400 speed at 45 deg
    let result = v.rotate_by(std::f32::consts::FRAC_PI_2);
    assert!(result.0.x > 0.0, "should have positive x (lower-right)");
    assert!(result.0.y < 0.0, "should have negative y (lower-right)");
    assert!(
        (result.speed() - v.speed()).abs() < 1e-1,
        "speed should be preserved"
    );
}

#[test]
fn rotate_by_from_diagonal_counterclockwise() {
    // Start at 45 degrees (upper-right), rotate 90 counterclockwise → upper-left
    let v = Velocity2D(Vec2::new(283.0, 283.0));
    let result = v.rotate_by(-std::f32::consts::FRAC_PI_2);
    assert!(result.0.x < 0.0, "should have negative x (upper-left)");
    assert!(result.0.y > 0.0, "should have positive y (upper-left)");
}
