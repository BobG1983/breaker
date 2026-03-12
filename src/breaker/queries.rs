//! Breaker domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::breaker::components::{
    BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration, BreakerMaxSpeed,
    BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BreakerWidth, BumpEarlyWindow,
    BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown,
    DashDuration, DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, SettleDuration,
    SettleTiltEase,
};

/// Breaker movement data — position, velocity, speed limits, and playfield clamping.
pub type BreakerMovementQuery = (
    &'static mut Transform,
    &'static mut BreakerVelocity,
    &'static BreakerState,
    &'static BreakerMaxSpeed,
    &'static BreakerAcceleration,
    &'static BreakerDeceleration,
    &'static DecelEasing,
    &'static BreakerWidth,
);

/// Breaker dash state machine data — full state, velocity, tilt, and all timing params.
pub type BreakerDashQuery = (
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
pub type BreakerResetQuery = (
    &'static mut Transform,
    &'static mut BreakerState,
    &'static mut BreakerVelocity,
    &'static mut BreakerTilt,
    &'static mut BreakerStateTimer,
    &'static BreakerBaseY,
);

/// Bump timing window data — state and all timing/cooldown params.
pub type BumpTimingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
);

/// Breaker bump telemetry — state, bump, tilt, velocity, and window sizes.
#[cfg(feature = "dev")]
pub type BreakerBumpTelemetryQuery = (
    &'static BreakerState,
    &'static BumpState,
    &'static BreakerTilt,
    &'static BreakerVelocity,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
);
