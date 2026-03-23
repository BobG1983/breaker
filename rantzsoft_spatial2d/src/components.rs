//! Core spatial components: `Position2D`, `Rotation2D`, `Scale2D`, and related types.

use std::ops::{Add, Div, Mul, Sub};

use bevy::prelude::*;

use crate::propagation::{PositionPropagation, RotationPropagation, ScalePropagation};

/// 2D world-space position.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct Position2D(pub Vec2);

impl Position2D {
    /// Euclidean distance to another position.
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        self.0.distance(other.0)
    }

    /// Squared euclidean distance to another position (avoids sqrt).
    #[must_use]
    pub fn distance_squared(&self, other: &Self) -> f32 {
        self.0.distance_squared(other.0)
    }
}

impl Add<Vec2> for Position2D {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<Vec2> for Position2D {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Mul<f32> for Position2D {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for Position2D {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

/// 2D rotation backed by Bevy's `Rot2`.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Rotation2D(pub Rot2);

impl Default for Rotation2D {
    fn default() -> Self {
        Self(Rot2::IDENTITY)
    }
}

impl Rotation2D {
    /// Creates a `Rotation2D` from degrees.
    #[must_use]
    pub fn from_degrees(degrees: f32) -> Self {
        Self(Rot2::degrees(degrees))
    }

    /// Creates a `Rotation2D` from radians.
    #[must_use]
    pub fn from_radians(radians: f32) -> Self {
        Self(Rot2::radians(radians))
    }

    /// Returns the rotation in radians.
    #[must_use]
    pub fn as_radians(&self) -> f32 {
        self.0.as_radians()
    }

    /// Returns the rotation in degrees.
    #[must_use]
    pub fn as_degrees(&self) -> f32 {
        self.0.as_degrees()
    }

    /// Converts to a 3D quaternion suitable for `Transform.rotation`.
    #[must_use]
    pub fn to_quat(&self) -> Quat {
        Quat::from_rotation_z(self.0.as_radians())
    }
}

/// Non-uniform 2D scale. Both components must be non-zero.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Scale2D {
    /// Horizontal scale factor.
    pub x: f32,
    /// Vertical scale factor.
    pub y: f32,
}

impl Default for Scale2D {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl Scale2D {
    /// Creates a new `Scale2D`.
    ///
    /// # Panics
    /// Panics if `x` or `y` is zero (within [`f32::EPSILON`]).
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        assert!(
            x.abs() > f32::EPSILON && y.abs() > f32::EPSILON,
            "Scale2D components must be non-zero"
        );
        Self { x, y }
    }

    /// Creates a uniform `Scale2D` with equal x and y.
    #[must_use]
    pub fn uniform(value: f32) -> Self {
        Self::new(value, value)
    }

    /// Converts to a `Vec3` suitable for `Transform.scale` (z = 1.0).
    #[must_use]
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, 1.0)
    }
}

/// Snapshot of the previous frame's position for interpolation.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct PreviousPosition(pub Vec2);

/// Snapshot of the previous frame's rotation for interpolation.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PreviousRotation(pub Rot2);

impl Default for PreviousRotation {
    fn default() -> Self {
        Self(Rot2::IDENTITY)
    }
}

/// Marker: this entity interpolates its `Transform` between fixed timesteps.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct InterpolateTransform2D;

/// Visual pixel offset applied after propagation (e.g., for screen shake).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct VisualOffset(pub Vec3);

/// Marker that requires all spatial components via Bevy's required components.
#[derive(Component, Debug, Default)]
#[require(
    Position2D,
    Rotation2D,
    Scale2D,
    PreviousPosition,
    PreviousRotation,
    PositionPropagation,
    RotationPropagation,
    ScalePropagation
)]
pub struct Spatial2D;

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

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
    }
}
