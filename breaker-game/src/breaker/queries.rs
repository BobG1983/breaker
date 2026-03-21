//! Breaker domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerHeight, BreakerMaxSpeed, BreakerState, BreakerStateTimer, BreakerTilt,
        BreakerVelocity, BreakerWidth, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown,
        BumpPerfectWindow, BumpState, BumpWeakCooldown, DashDuration, DashSpeedMultiplier,
        DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
    },
    chips::components::{BreakerSpeedBoost, WidthBoost},
    interpolate::components::PhysicsTranslation,
    shared::EntityScale,
};

/// Breaker movement data — position, velocity, speed limits, and playfield clamping.
pub(crate) type MovementQuery = (
    &'static mut Transform,
    &'static mut BreakerVelocity,
    &'static BreakerState,
    &'static BreakerMaxSpeed,
    &'static BreakerAcceleration,
    &'static BreakerDeceleration,
    &'static DecelEasing,
    &'static BreakerWidth,
    Option<&'static BreakerSpeedBoost>,
    Option<&'static WidthBoost>,
);

/// Breaker dash state machine data — full state, velocity, tilt, and all timing params.
pub(crate) type DashQuery = (
    &'static mut BreakerState,
    &'static mut BreakerVelocity,
    &'static mut BreakerTilt,
    &'static mut BreakerStateTimer,
    &'static BreakerMaxSpeed,
    &'static BreakerDeceleration,
    &'static DecelEasing,
    &'static DashSpeedMultiplier,
    &'static DashDuration,
    &'static DashTilt,
    &'static DashTiltEase,
    &'static BrakeTilt,
    &'static BrakeDecel,
    &'static SettleDuration,
    &'static SettleTiltEase,
);

/// Breaker reset data — mutable state cleared at node start.
pub(crate) type ResetQuery = (
    &'static mut Transform,
    &'static mut BreakerState,
    &'static mut BreakerVelocity,
    &'static mut BreakerTilt,
    &'static mut BreakerStateTimer,
    &'static mut BumpState,
    &'static BreakerBaseY,
    Option<&'static mut PhysicsTranslation>,
);

/// Bump timing window data — state, timing/cooldown params.
pub(crate) type BumpTimingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
);

/// Bump grading data — state, timing windows, and cooldowns for `grade_bump`.
pub(crate) type BumpGradingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
);

/// Breaker data needed by the width boost visual system.
pub(crate) type WidthBoostVisualQuery = (
    &'static BreakerWidth,
    Option<&'static WidthBoost>,
    &'static BreakerHeight,
    Option<&'static EntityScale>,
    &'static mut Transform,
);

/// Breaker bump telemetry — state, bump, tilt, velocity, and window sizes.
#[cfg(feature = "dev")]
pub type BumpTelemetryQuery = (
    &'static BreakerState,
    &'static BumpState,
    &'static BreakerTilt,
    &'static BreakerVelocity,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
);
