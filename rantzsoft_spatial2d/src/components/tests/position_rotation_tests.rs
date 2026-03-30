use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::super::definitions::*;

// ── Position2D ──────────────────────────────────────────────

#[test]
fn position_default_is_zero() {
    assert_eq!(Position2D::default().0, Vec2::ZERO);
}

#[test]
fn position_add_vec2() {
    let result = Position2D(Vec2::new(1.0, 2.0)) + Vec2::new(3.0, 4.0);
    assert_eq!(result, Position2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn position_sub_vec2() {
    let result = Position2D(Vec2::new(5.0, 5.0)) - Vec2::new(1.0, 2.0);
    assert_eq!(result, Position2D(Vec2::new(4.0, 3.0)));
}

#[test]
fn position_mul_f32() {
    let result = Position2D(Vec2::new(2.0, 3.0)) * 2.0;
    assert_eq!(result, Position2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn position_div_f32() {
    let result = Position2D(Vec2::new(6.0, 8.0)) / 2.0;
    assert_eq!(result, Position2D(Vec2::new(3.0, 4.0)));
}

#[test]
fn position_distance() {
    let a = Position2D(Vec2::ZERO);
    let b = Position2D(Vec2::new(3.0, 4.0));
    assert!((a.distance(&b) - 5.0).abs() < f32::EPSILON);
}

#[test]
fn position_distance_squared() {
    let a = Position2D(Vec2::ZERO);
    let b = Position2D(Vec2::new(3.0, 4.0));
    assert!((a.distance_squared(&b) - 25.0).abs() < f32::EPSILON);
}

// ── Rotation2D ──────────────────────────────────────────────

#[test]
fn rotation_default_is_zero_radians() {
    assert!((Rotation2D::default().as_radians()).abs() < 1e-6);
}

#[test]
fn rotation_from_degrees_round_trip() {
    let rot = Rotation2D::from_degrees(90.0);
    assert!(
        (rot.as_radians() - FRAC_PI_2).abs() < 1e-6,
        "expected {} but got {}",
        FRAC_PI_2,
        rot.as_radians()
    );
}

#[test]
fn rotation_from_radians_round_trip() {
    let rot = Rotation2D::from_radians(PI);
    assert!(
        (rot.as_degrees().abs() - 180.0).abs() < 1e-4,
        "expected 180.0 but got {}",
        rot.as_degrees()
    );
}

#[test]
fn rotation_to_quat_ninety_degrees() {
    let rot = Rotation2D::from_degrees(90.0);
    let expected = Quat::from_rotation_z(FRAC_PI_2);
    let actual = rot.to_quat();
    assert!(
        (actual.x - expected.x).abs() < 1e-6
            && (actual.y - expected.y).abs() < 1e-6
            && (actual.z - expected.z).abs() < 1e-6
            && (actual.w - expected.w).abs() < 1e-6,
        "expected {expected:?} but got {actual:?}"
    );
}

#[test]
fn rotation_default_to_quat_is_identity() {
    let actual = Rotation2D::default().to_quat();
    let expected = Quat::IDENTITY;
    assert!(
        (actual.x - expected.x).abs() < 1e-6
            && (actual.y - expected.y).abs() < 1e-6
            && (actual.z - expected.z).abs() < 1e-6
            && (actual.w - expected.w).abs() < 1e-6,
        "expected identity {expected:?} but got {actual:?}"
    );
}
