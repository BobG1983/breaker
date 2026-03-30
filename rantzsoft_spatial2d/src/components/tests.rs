use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::definitions::*;
use crate::propagation::{PositionPropagation, RotationPropagation, ScalePropagation};

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

// ── Scale2D ─────────────────────────────────────────────────

#[test]
fn scale_default_is_uniform_one() {
    let s = Scale2D::default();
    assert!((s.x - 1.0).abs() < f32::EPSILON);
    assert!((s.y - 1.0).abs() < f32::EPSILON);
}

#[test]
fn scale_new_stores_values() {
    let s = Scale2D::new(2.0, 3.0);
    assert!((s.x - 2.0).abs() < f32::EPSILON);
    assert!((s.y - 3.0).abs() < f32::EPSILON);
}

#[test]
#[should_panic(expected = "Scale2D components must be non-zero")]
fn scale_new_panics_on_zero_x() {
    let _ = Scale2D::new(0.0, 1.0);
}

#[test]
#[should_panic(expected = "Scale2D components must be non-zero")]
fn scale_new_panics_on_zero_y() {
    let _ = Scale2D::new(1.0, 0.0);
}

#[test]
fn scale_uniform() {
    let s = Scale2D::uniform(2.0);
    assert_eq!(s, Scale2D { x: 2.0, y: 2.0 });
}

#[test]
fn scale_to_vec3() {
    let v = Scale2D::new(2.0, 3.0).to_vec3();
    assert_eq!(v, Vec3::new(2.0, 3.0, 1.0));
}

// ── Simple wrappers ─────────────────────────────────────────

#[test]
fn previous_position_default_is_zero() {
    assert_eq!(PreviousPosition::default().0, Vec2::ZERO);
}

#[test]
fn previous_scale_default_is_one_one() {
    let ps = PreviousScale::default();
    assert!((ps.x - 1.0).abs() < f32::EPSILON);
    assert!((ps.y - 1.0).abs() < f32::EPSILON);
}

#[test]
fn previous_rotation_default_is_zero() {
    assert!(PreviousRotation::default().0.as_radians().abs() < 1e-6);
}

#[test]
fn visual_offset_default_is_zero() {
    assert_eq!(VisualOffset::default().0, Vec3::ZERO);
}

#[test]
fn interpolate_transform_marker_is_component() {
    // Verify `InterpolateTransform2D` can be used as a component.
    // If this compiles and the entity spawns, the marker works.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    let entity = app.world_mut().spawn(InterpolateTransform2D).id();
    assert!(app.world().get::<InterpolateTransform2D>(entity).is_some());
}

// ── ApplyVelocity ────────────────────────────────────────────

#[test]
fn apply_velocity_marker_is_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    let entity = app.world_mut().spawn(ApplyVelocity).id();
    assert!(app.world().get::<ApplyVelocity>(entity).is_some());
}

// ── Velocity2D ─────────────────────────────────────────────

#[test]
fn velocity_default_is_zero() {
    assert_eq!(Velocity2D::default().0, Vec2::ZERO);
}

#[test]
fn velocity_speed_returns_magnitude() {
    assert!(
        (Velocity2D(Vec2::new(3.0, 4.0)).speed() - 5.0).abs() < f32::EPSILON,
        "speed of (3, 4) should be 5.0"
    );
}

#[test]
fn velocity_speed_zero_returns_zero() {
    assert!(
        Velocity2D(Vec2::ZERO).speed().abs() < f32::EPSILON,
        "speed of zero velocity should be 0.0"
    );
}

#[test]
fn velocity_clamped_high_to_max() {
    let v = Velocity2D(Vec2::new(0.0, 800.0)).clamped(200.0, 600.0);
    let speed = v.0.length();
    assert!(
        (speed - 600.0).abs() < 1e-3,
        "magnitude should be clamped to 600.0, got {speed}"
    );
    // Direction should be preserved (pointing up).
    let dir = v.0.normalize();
    assert!(
        (dir.y - 1.0).abs() < 1e-5,
        "direction should be (0, 1), got {dir:?}"
    );
}

#[test]
fn velocity_clamped_exactly_at_max_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 600.0)).clamped(200.0, 600.0);
    assert!(
        (v.0.length() - 600.0).abs() < 1e-3,
        "velocity exactly at max should remain unchanged"
    );
}

#[test]
fn velocity_clamped_low_to_min() {
    let v = Velocity2D(Vec2::new(0.0, 100.0)).clamped(200.0, 600.0);
    let speed = v.0.length();
    assert!(
        (speed - 200.0).abs() < 1e-3,
        "magnitude should be clamped up to 200.0, got {speed}"
    );
    let dir = v.0.normalize();
    assert!(
        (dir.y - 1.0).abs() < 1e-5,
        "direction should be preserved as (0, 1), got {dir:?}"
    );
}

#[test]
fn velocity_clamped_exactly_at_min_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 200.0)).clamped(200.0, 600.0);
    assert!(
        (v.0.length() - 200.0).abs() < 1e-3,
        "velocity exactly at min should remain unchanged"
    );
}

#[test]
fn velocity_clamped_zero_returns_zero() {
    let v = Velocity2D(Vec2::ZERO).clamped(200.0, 600.0);
    assert_eq!(v, Velocity2D(Vec2::ZERO), "zero velocity should stay zero");
}

#[test]
fn velocity_add_vec2() {
    let result = Velocity2D(Vec2::new(1.0, 2.0)) + Vec2::new(3.0, 4.0);
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn velocity_sub_vec2() {
    let result = Velocity2D(Vec2::new(5.0, 5.0)) - Vec2::new(1.0, 2.0);
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 3.0)));
}

#[test]
fn velocity_mul_f32() {
    let result = Velocity2D(Vec2::new(2.0, 3.0)) * 2.0;
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn velocity_div_f32() {
    let result = Velocity2D(Vec2::new(6.0, 8.0)) / 2.0;
    assert_eq!(result, Velocity2D(Vec2::new(3.0, 4.0)));
}

// ── PreviousVelocity ────────────────────────────────────────

#[test]
fn previous_velocity_default_is_zero() {
    assert_eq!(PreviousVelocity::default().0, Vec2::ZERO);
}

// ── GlobalPosition2D ────────────────────────────────────────

#[test]
fn global_position_default_is_zero() {
    assert_eq!(GlobalPosition2D::default().0, Vec2::ZERO);
}

// ── GlobalRotation2D ────────────────────────────────────────

#[test]
fn global_rotation_default_is_identity() {
    assert!(
        GlobalRotation2D::default().0.as_radians().abs() < 1e-6,
        "GlobalRotation2D default should be identity (0 radians)"
    );
}

// ── GlobalScale2D ───────────────────────────────────────────

#[test]
fn global_scale_default_is_one_one() {
    let s = GlobalScale2D::default();
    assert!(
        (s.x - 1.0).abs() < f32::EPSILON,
        "GlobalScale2D.x should default to 1.0, got {}",
        s.x
    );
    assert!(
        (s.y - 1.0).abs() < f32::EPSILON,
        "GlobalScale2D.y should default to 1.0, got {}",
        s.y
    );
}

// ── Spatial2D required components ───────────────────────────

#[test]
fn spatial2d_adds_all_required_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Spatial2D).id();
    // Process required components.
    app.update();

    let world = app.world();
    assert!(
        world.get::<Position2D>(entity).is_some(),
        "missing Position2D"
    );
    assert!(
        world.get::<Rotation2D>(entity).is_some(),
        "missing Rotation2D"
    );
    assert!(world.get::<Scale2D>(entity).is_some(), "missing Scale2D");
    assert!(
        world.get::<PreviousPosition>(entity).is_some(),
        "missing PreviousPosition"
    );
    assert!(
        world.get::<PreviousRotation>(entity).is_some(),
        "missing PreviousRotation"
    );
    assert!(
        world.get::<PreviousScale>(entity).is_some(),
        "missing PreviousScale"
    );
    // Verify PreviousScale defaults to (1.0, 1.0).
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 1.0).abs() < f32::EPSILON,
        "PreviousScale.x should default to 1.0, got {}",
        prev_scale.x
    );
    assert!(
        (prev_scale.y - 1.0).abs() < f32::EPSILON,
        "PreviousScale.y should default to 1.0, got {}",
        prev_scale.y
    );
    assert!(
        world.get::<PositionPropagation>(entity).is_some(),
        "missing PositionPropagation"
    );
    assert!(
        world.get::<RotationPropagation>(entity).is_some(),
        "missing RotationPropagation"
    );
    assert!(
        world.get::<ScalePropagation>(entity).is_some(),
        "missing ScalePropagation"
    );

    // Verify propagation defaults are Relative.
    assert_eq!(
        *world.get::<PositionPropagation>(entity).unwrap(),
        PositionPropagation::Relative,
        "PositionPropagation should default to Relative"
    );
    assert_eq!(
        *world.get::<RotationPropagation>(entity).unwrap(),
        RotationPropagation::Relative,
        "RotationPropagation should default to Relative"
    );
    assert_eq!(
        *world.get::<ScalePropagation>(entity).unwrap(),
        ScalePropagation::Relative,
        "ScalePropagation should default to Relative"
    );

    // Verify Global* components are required by Spatial2D.
    assert!(
        world.get::<GlobalPosition2D>(entity).is_some(),
        "missing GlobalPosition2D"
    );
    assert!(
        world.get::<GlobalRotation2D>(entity).is_some(),
        "missing GlobalRotation2D"
    );
    assert!(
        world.get::<GlobalScale2D>(entity).is_some(),
        "missing GlobalScale2D"
    );
}

#[test]
fn spatial2d_does_not_require_velocity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let entity = app.world_mut().spawn(Spatial2D).id();
    app.update();

    let world = app.world();
    assert!(
        world.get::<Velocity2D>(entity).is_none(),
        "Spatial2D should NOT require Velocity2D"
    );
    assert!(
        world.get::<PreviousVelocity>(entity).is_none(),
        "Spatial2D should NOT require PreviousVelocity"
    );
}

// ── D1: Velocity2D::clamped() preserves direction for diagonal velocity ──

#[test]
fn velocity_clamped_preserves_direction_diagonal() {
    // Positive diagonal: magnitude 500.0, direction (0.6, 0.8)
    let v = Velocity2D(Vec2::new(300.0, 400.0));
    let result = v.clamped(200.0, 400.0);

    let speed = result.0.length();
    assert!(
        (speed - 400.0).abs() < 1e-3,
        "magnitude should be clamped to 400.0, got {speed}"
    );

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

    // Edge case: negative diagonal, same magnitude 500.0, clamped to 400.0
    let v_neg = Velocity2D(Vec2::new(-300.0, -400.0));
    let result_neg = v_neg.clamped(200.0, 400.0);

    let speed_neg = result_neg.0.length();
    assert!(
        (speed_neg - 400.0).abs() < 1e-3,
        "negative diagonal magnitude should be clamped to 400.0, got {speed_neg}"
    );

    let dir_neg = result_neg.0.normalize();
    assert!(
        (dir_neg.x - (-0.6)).abs() < 1e-5,
        "negative direction x should be -0.6, got {}",
        dir_neg.x
    );
    assert!(
        (dir_neg.y - (-0.8)).abs() < 1e-5,
        "negative direction y should be -0.8, got {}",
        dir_neg.y
    );
}
