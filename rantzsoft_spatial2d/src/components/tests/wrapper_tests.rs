use bevy::prelude::*;

use super::super::definitions::*;
use crate::propagation::{PositionPropagation, RotationPropagation, ScalePropagation};

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
