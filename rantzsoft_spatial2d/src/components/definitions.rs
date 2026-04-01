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
    /// Creates a velocity rotated `angle` radians from straight up.
    ///
    /// Positive angle = clockwise (rightward), negative = counterclockwise (leftward).
    /// - `0.0` → `(0, speed)` (straight up)
    /// - `PI/4` → `(speed * 0.707, speed * 0.707)` (upper-right)
    /// - `-PI/4` → `(-speed * 0.707, speed * 0.707)` (upper-left)
    #[must_use]
    pub fn from_angle_up(angle: f32, speed: f32) -> Self {
        Self(Vec2::new(speed * angle.sin(), speed * angle.cos()))
    }

    /// Rotates this velocity clockwise by `angle` radians. Preserves speed.
    ///
    /// Positive angle = clockwise, negative = counterclockwise.
    #[must_use]
    pub fn rotate_by(&self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self(Vec2::new(
            self.0.y.mul_add(sin, self.0.x * cos),
            self.0.y.mul_add(cos, -self.0.x * sin),
        ))
    }

    /// Returns the speed (magnitude) of this velocity.
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.0.length()
    }

    /// Returns a new `Velocity2D` with the same direction but the given speed.
    /// Zero velocity returns zero.
    #[must_use]
    pub fn with_speed(&self, speed: f32) -> Self {
        let current = self.0.length();
        if current < f32::EPSILON {
            return *self;
        }
        Self(self.0 * (speed / current))
    }

    /// Clamps magnitude between `min_speed` and `max_speed`, preserving direction.
    /// If below min, normalizes and multiplies by min. If above max, normalizes
    /// and multiplies by max. Otherwise returns unchanged. Zero velocity returns zero.
    #[must_use]
    pub fn clamp(&self, min_speed: f32, max_speed: f32) -> Self {
        let speed = self.0.length();
        if speed < f32::EPSILON {
            return *self;
        }
        if speed < min_speed {
            self.with_speed(min_speed)
        } else if speed > max_speed {
            self.with_speed(max_speed)
        } else {
            *self
        }
    }

    /// Clamps velocity direction so it stays at least `bounds.0` from
    /// horizontal and at least `bounds.1` from vertical. Preserves speed
    /// and axis signs. Zero velocity is returned unchanged.
    #[must_use]
    pub fn clamp_angle(&self, bounds: (f32, f32)) -> Self {
        let speed = self.0.length();
        if speed < f32::EPSILON {
            return *self;
        }

        let angle_from_horizontal = self.0.y.abs().atan2(self.0.x.abs());
        let upper = (std::f32::consts::FRAC_PI_2 - bounds.1).max(bounds.0);
        let clamped = angle_from_horizontal.clamp(bounds.0, upper);

        if (clamped - angle_from_horizontal).abs() > f32::EPSILON {
            let sign_x = if self.0.x.abs() < f32::EPSILON {
                1.0
            } else {
                self.0.x.signum()
            };
            let sign_y = if self.0.y.abs() < f32::EPSILON {
                1.0
            } else {
                self.0.y.signum()
            };
            Self(Vec2::new(
                sign_x * speed * clamped.cos(),
                sign_y * speed * clamped.sin(),
            ))
        } else {
            *self
        }
    }

    /// Applies velocity constraints from spatial components: clamps angle then
    /// sets speed to `base_speed.clamp(min, max)`. Optional parameters degrade
    /// gracefully — `None` means no constraint for that axis/bound.
    #[must_use]
    pub fn constrained(
        &self,
        base_speed: &BaseSpeed,
        min_speed: Option<&MinSpeed>,
        max_speed: Option<&MaxSpeed>,
        min_angle_h: Option<&MinAngleHorizontal>,
        min_angle_v: Option<&MinAngleVertical>,
    ) -> Self {
        let angle_h = min_angle_h.map_or(0.0, |a| a.0);
        let angle_v = min_angle_v.map_or(0.0, |a| a.0);
        let min = min_speed.map_or(0.0, |s| s.0);
        let max = max_speed.map_or(f32::MAX, |s| s.0);
        self.clamp_angle((angle_h, angle_v))
            .with_speed(base_speed.0.clamp(min, max))
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

/// Base speed of an entity. The natural speed before any multipliers are applied.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct BaseSpeed(pub f32);

impl From<f32> for BaseSpeed {
    fn from(v: f32) -> Self {
        Self(v)
    }
}
impl From<BaseSpeed> for f32 {
    fn from(v: BaseSpeed) -> Self {
        v.0
    }
}

/// Minimum speed constraint. Entity velocity should not drop below this magnitude.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct MinSpeed(pub f32);

impl From<f32> for MinSpeed {
    fn from(v: f32) -> Self {
        Self(v)
    }
}
impl From<MinSpeed> for f32 {
    fn from(v: MinSpeed) -> Self {
        v.0
    }
}

/// Maximum speed constraint. Entity velocity should not exceed this magnitude.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct MaxSpeed(pub f32);

impl From<f32> for MaxSpeed {
    fn from(v: f32) -> Self {
        Self(v)
    }
}
impl From<MaxSpeed> for f32 {
    fn from(v: MaxSpeed) -> Self {
        v.0
    }
}

/// Minimum angle from horizontal in radians. Velocity direction should stay
/// at least this far from the horizontal axis.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct MinAngleHorizontal(pub f32);

impl From<f32> for MinAngleHorizontal {
    fn from(v: f32) -> Self {
        Self(v)
    }
}
impl From<MinAngleHorizontal> for f32 {
    fn from(v: MinAngleHorizontal) -> Self {
        v.0
    }
}

/// Minimum angle from vertical in radians. Velocity direction should stay
/// at least this far from the vertical axis.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
pub struct MinAngleVertical(pub f32);

impl From<f32> for MinAngleVertical {
    fn from(v: f32) -> Self {
        Self(v)
    }
}
impl From<MinAngleVertical> for f32 {
    fn from(v: MinAngleVertical) -> Self {
        v.0
    }
}

/// Marker for entities with velocity constraint data ([`BaseSpeed`] and
/// optionally [`MinSpeed`], [`MaxSpeed`], [`MinAngleHorizontal`],
/// [`MinAngleVertical`]).
///
/// Use [`Spatial::builder()`] for a typestate builder that constructs
/// the correct component tuple.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Spatial;

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
