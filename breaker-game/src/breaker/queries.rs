//! Breaker domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerMaxSpeed, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity,
        BreakerWidth, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectMultiplier,
        BumpPerfectWindow, BumpState, BumpWeakCooldown, BumpWeakMultiplier, DashDuration,
        DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
    },
    chips::components::{BreakerSpeedBoost, BumpForceBoost, WidthBoost},
    interpolate::components::PhysicsTranslation,
};

/// Breaker movement data — position, velocity, speed limits, and playfield clamping.
pub type MovementQuery = (
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
pub type DashQuery = (
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
pub type ResetQuery = (
    &'static mut Transform,
    &'static mut BreakerState,
    &'static mut BreakerVelocity,
    &'static mut BreakerTilt,
    &'static mut BreakerStateTimer,
    &'static mut BumpState,
    &'static BreakerBaseY,
    Option<&'static mut PhysicsTranslation>,
);

/// Bump timing window data — state, timing/cooldown params, and velocity multipliers.
pub type BumpTimingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
    Option<&'static BumpPerfectMultiplier>,
    Option<&'static BumpWeakMultiplier>,
    Option<&'static BumpForceBoost>,
);

/// Bump grading data — state, timing windows, cooldowns, and multipliers for `grade_bump`.
pub type BumpGradingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
    Option<&'static BumpPerfectMultiplier>,
    Option<&'static BumpWeakMultiplier>,
    Option<&'static BumpForceBoost>,
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
