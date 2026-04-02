use bevy::prelude::*;

use super::super::super::definitions::*;

// ── constrained() ───────────────────────────────────────────────────────

#[test]
fn constrained_applies_base_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(&BaseSpeed(400.0), None, None, None, None);
    assert!((result.speed() - 400.0).abs() < 1e-3);
}

#[test]
fn constrained_clamps_to_min_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(
        &BaseSpeed(100.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        None,
        None,
    );
    assert!((result.speed() - 200.0).abs() < 1e-3);
}

#[test]
fn constrained_clamps_to_max_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(
        &BaseSpeed(1000.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        None,
        None,
    );
    assert!((result.speed() - 800.0).abs() < 1e-3);
}

#[test]
fn constrained_no_min_max_uses_permissive_defaults() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(&BaseSpeed(500.0), None, None, None, None);
    assert!((result.speed() - 500.0).abs() < 1e-3);
}

#[test]
fn constrained_applies_angle_clamping() {
    // Near-horizontal velocity should be clamped to min angle from horizontal
    let v = Velocity2D(Vec2::new(400.0, 1.0));
    let min_h = MinAngleHorizontal(0.3); // ~17 degrees
    let result = v.constrained(&BaseSpeed(400.0), None, None, Some(&min_h), None);
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        angle >= 0.3 - 1e-3,
        "angle from horizontal should be >= 0.3, got {angle}"
    );
}

#[test]
fn constrained_no_angle_constraints_preserves_direction() {
    let v = Velocity2D(Vec2::new(300.0, 400.0));
    let result = v.constrained(&BaseSpeed(500.0), None, None, None, None);
    let dir_before = v.0.normalize();
    let dir_after = result.0.normalize();
    assert!((dir_before.x - dir_after.x).abs() < 1e-3);
    assert!((dir_before.y - dir_after.y).abs() < 1e-3);
}

#[test]
fn constrained_zero_velocity_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.constrained(
        &BaseSpeed(400.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        Some(&MinAngleHorizontal(0.1)),
        Some(&MinAngleVertical(0.1)),
    );
    assert_eq!(result.0, Vec2::ZERO);
}

#[test]
fn constrained_all_params_applied() {
    let v = Velocity2D(Vec2::new(0.6, 0.8));
    let result = v.constrained(
        &BaseSpeed(400.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        Some(&MinAngleHorizontal(0.087)),
        Some(&MinAngleVertical(0.087)),
    );
    assert!(
        (result.speed() - 400.0).abs() < 1e-3,
        "speed should be base_speed"
    );
}
