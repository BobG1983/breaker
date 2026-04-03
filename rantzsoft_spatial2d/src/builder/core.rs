//! Typestate builder for spatial velocity constraint component bundles.
//!
//! Entry point: [`Spatial::builder()`]. The builder prevents invalid
//! combinations at compile time — `.with_speed()` and `.with_clamped_speed()`
//! are mutually exclusive, and `build()` is unavailable until at least one
//! speed method has been called.
//!
//! # Examples
//!
//! ```ignore
//! // Bolt: position + clamped speed + clamped angle
//! let bundle = Spatial::builder()
//!     .at_position(Vec2::new(100.0, 200.0))
//!     .with_clamped_speed(400.0, 200.0, 800.0)
//!     .with_clamped_angle(0.087, 0.087)
//!     .build();
//!
//! // Breaker: speed only, no clamping, default position
//! let bundle = Spatial::builder()
//!     .with_speed(600.0)
//!     .build();
//!
//! // Speed then optional clamping
//! let bundle = Spatial::builder()
//!     .with_speed(400.0)
//!     .clamped(200.0, 800.0)
//!     .build();
//! ```

use bevy::prelude::{Bundle, Vec2};

use crate::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Position2D,
    PreviousPosition, Spatial,
};

// ── Typestate markers ───────────────────────────────────────────────────────

/// No position configured — `Position2D` and `PreviousPosition` use defaults.
pub struct NoPosition;

/// Position configured via `.at_position()`.
pub struct HasPosition {
    pos: Vec2,
}

/// Speed has not been configured yet.
pub struct NoSpeed;

/// Base speed configured, no min/max clamping.
pub struct SpeedOnly {
    base: f32,
}

/// Base speed configured with min/max clamping bounds.
pub struct ClampedSpeed {
    base: f32,
    min: f32,
    max: f32,
}

/// No angle constraint configured.
pub struct NoAngle;

/// Angle constraints configured.
pub struct WithAngle {
    h: f32,
    v: f32,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Typestate builder for spatial velocity constraint components.
///
/// Created via [`Spatial::builder()`]. Call speed and angle methods to
/// configure, then `.build()` to get the component tuple.
pub struct SpatialDataBuilder<Position, Speed, Angle> {
    position: Position,
    speed: Speed,
    angle: Angle,
}

impl Spatial {
    /// Creates a [`SpatialDataBuilder`] in the unconfigured state.
    #[must_use]
    pub const fn builder() -> SpatialDataBuilder<NoPosition, NoSpeed, NoAngle> {
        SpatialDataBuilder {
            position: NoPosition,
            speed: NoSpeed,
            angle: NoAngle,
        }
    }
}

// ── Position transition ─────────────────────────────────────────────────────

impl<S, A> SpatialDataBuilder<NoPosition, S, A> {
    /// Sets initial position. `build()` will include `Position2D(pos)` and
    /// `PreviousPosition(pos)` in the output.
    pub fn at_position(self, pos: Vec2) -> SpatialDataBuilder<HasPosition, S, A> {
        SpatialDataBuilder {
            position: HasPosition { pos },
            speed: self.speed,
            angle: self.angle,
        }
    }
}

// ── Speed transitions (only from NoSpeed) ───────────────────────────────────

impl<P, A> SpatialDataBuilder<P, NoSpeed, A> {
    /// Sets base speed only. Use `.clamped(min, max)` to add bounds.
    pub fn with_speed(self, base: f32) -> SpatialDataBuilder<P, SpeedOnly, A> {
        SpatialDataBuilder {
            position: self.position,
            speed: SpeedOnly { base },
            angle: self.angle,
        }
    }

    /// Sets base speed with min/max bounds in one call.
    pub fn with_clamped_speed(
        self,
        base: f32,
        min: f32,
        max: f32,
    ) -> SpatialDataBuilder<P, ClampedSpeed, A> {
        SpatialDataBuilder {
            position: self.position,
            speed: ClampedSpeed { base, min, max },
            angle: self.angle,
        }
    }
}

// ── Optional clamping after with_speed() ────────────────────────────────────

impl<P, A> SpatialDataBuilder<P, SpeedOnly, A> {
    /// Adds min/max speed bounds after `.with_speed()`.
    pub fn clamped(self, min: f32, max: f32) -> SpatialDataBuilder<P, ClampedSpeed, A> {
        SpatialDataBuilder {
            position: self.position,
            speed: ClampedSpeed {
                base: self.speed.base,
                min,
                max,
            },
            angle: self.angle,
        }
    }
}

// ── Angle transition (from any speed state) ─────────────────────────────────

impl<P, S> SpatialDataBuilder<P, S, NoAngle> {
    /// Sets minimum angle constraints from horizontal and vertical.
    pub fn with_clamped_angle(self, h: f32, v: f32) -> SpatialDataBuilder<P, S, WithAngle> {
        SpatialDataBuilder {
            position: self.position,
            speed: self.speed,
            angle: WithAngle { h, v },
        }
    }
}

// ── Position helpers ────────────────────────────────────────────────────────

const fn position_components(pos: Vec2) -> (Position2D, PreviousPosition) {
    (Position2D(pos), PreviousPosition(pos))
}

// ── Build impls: Position-only (no speed/angle) ────────────────────────────
// Walls and other static entities need position but no velocity constraints.

impl SpatialDataBuilder<HasPosition, NoSpeed, NoAngle> {
    /// Builds `(Spatial, Position2D, PreviousPosition)` — position only, no speed.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(self.position.pos);
        (Spatial, pos.0, pos.1)
    }
}

// ── Build impls: NoPosition ─────────────────────────────────────────────────
// NoSpeed + NoAngle intentionally has NO build() — compile error if nothing configured.

impl SpatialDataBuilder<NoPosition, SpeedOnly, NoAngle> {
    /// Builds `(Spatial, BaseSpeed, Position2D, PreviousPosition)`.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(Vec2::ZERO);
        (Spatial, BaseSpeed(self.speed.base), pos.0, pos.1)
    }
}

impl SpatialDataBuilder<NoPosition, ClampedSpeed, NoAngle> {
    /// Builds spatial + speed clamp components with default position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(Vec2::ZERO);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinSpeed(self.speed.min),
            MaxSpeed(self.speed.max),
            pos.0,
            pos.1,
        )
    }
}

impl SpatialDataBuilder<NoPosition, SpeedOnly, WithAngle> {
    /// Builds spatial + speed + angle components with default position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(Vec2::ZERO);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinAngleHorizontal(self.angle.h),
            MinAngleVertical(self.angle.v),
            pos.0,
            pos.1,
        )
    }
}

impl SpatialDataBuilder<NoPosition, ClampedSpeed, WithAngle> {
    /// Builds all spatial constraint components with default position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(Vec2::ZERO);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinSpeed(self.speed.min),
            MaxSpeed(self.speed.max),
            MinAngleHorizontal(self.angle.h),
            MinAngleVertical(self.angle.v),
            pos.0,
            pos.1,
        )
    }
}

// ── Build impls: HasPosition ────────────────────────────────────────────────

impl SpatialDataBuilder<HasPosition, SpeedOnly, NoAngle> {
    /// Builds `(Spatial, BaseSpeed, Position2D, PreviousPosition)`.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(self.position.pos);
        (Spatial, BaseSpeed(self.speed.base), pos.0, pos.1)
    }
}

impl SpatialDataBuilder<HasPosition, ClampedSpeed, NoAngle> {
    /// Builds spatial + speed clamp components with explicit position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(self.position.pos);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinSpeed(self.speed.min),
            MaxSpeed(self.speed.max),
            pos.0,
            pos.1,
        )
    }
}

impl SpatialDataBuilder<HasPosition, SpeedOnly, WithAngle> {
    /// Builds spatial + speed + angle components with explicit position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(self.position.pos);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinAngleHorizontal(self.angle.h),
            MinAngleVertical(self.angle.v),
            pos.0,
            pos.1,
        )
    }
}

impl SpatialDataBuilder<HasPosition, ClampedSpeed, WithAngle> {
    /// Builds all spatial constraint components with explicit position.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let pos = position_components(self.position.pos);
        (
            Spatial,
            BaseSpeed(self.speed.base),
            MinSpeed(self.speed.min),
            MaxSpeed(self.speed.max),
            MinAngleHorizontal(self.angle.h),
            MinAngleVertical(self.angle.v),
            pos.0,
            pos.1,
        )
    }
}
