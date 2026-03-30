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
    pub const fn to_vec3(self) -> Vec3 {
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

/// Snapshot of the previous frame's scale for interpolation.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct PreviousScale {
    /// Previous horizontal scale factor.
    pub x: f32,
    /// Previous vertical scale factor.
    pub y: f32,
}

impl Default for PreviousScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// Marker: this entity interpolates its `Transform` between fixed timesteps.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct InterpolateTransform2D;

/// Visual pixel offset applied after propagation (e.g., for screen shake).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct VisualOffset(pub Vec3);

/// 2D velocity vector.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct Velocity2D(pub Vec2);

impl Velocity2D {
    /// Returns the speed (magnitude) of this velocity.
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.0.length()
    }

    /// Returns a new `Velocity2D` with magnitude clamped between `min_speed`
    /// and `max_speed`, preserving direction. Zero velocity returns zero.
    #[must_use]
    pub fn clamped(&self, min_speed: f32, max_speed: f32) -> Self {
        let speed = self.0.length();
        if speed < f32::EPSILON {
            return *self;
        }
        let clamped_speed = speed.clamp(min_speed, max_speed);
        Self(self.0 * (clamped_speed / speed))
    }
}

impl Add<Vec2> for Velocity2D {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<Vec2> for Velocity2D {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Mul<f32> for Velocity2D {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for Velocity2D {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

/// Marker: entities with this component have their `Position2D` advanced by
/// `Velocity2D` each fixed tick via [`apply_velocity`].
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
pub struct ApplyVelocity;

/// Snapshot of the previous frame's velocity for interpolation.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct PreviousVelocity(pub Vec2);

/// Global 2D position computed from parent hierarchy.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect, Deref, DerefMut)]
pub struct GlobalPosition2D(pub Vec2);

/// Global 2D rotation computed from parent hierarchy.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct GlobalRotation2D(pub Rot2);

impl Default for GlobalRotation2D {
    fn default() -> Self {
        Self(Rot2::IDENTITY)
    }
}

/// Global non-uniform 2D scale computed from parent hierarchy.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct GlobalScale2D {
    /// Global horizontal scale factor.
    pub x: f32,
    /// Global vertical scale factor.
    pub y: f32,
}

impl Default for GlobalScale2D {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// Marker that requires all spatial components via Bevy's required components.
#[derive(Component, Debug, Default)]
#[require(
    Position2D,
    Rotation2D,
    Scale2D,
    PreviousPosition,
    PreviousRotation,
    PreviousScale,
    GlobalPosition2D,
    GlobalRotation2D,
    GlobalScale2D,
    PositionPropagation,
    RotationPropagation,
    ScalePropagation,
    Transform
)]
pub struct Spatial2D;
