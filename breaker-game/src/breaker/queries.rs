//! Breaker domain query type aliases — clippy `type_complexity` lint.

use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, PreviousPosition, Scale2D};

use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerHeight, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BreakerWidth,
        BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState,
        BumpWeakCooldown, DashDuration, DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing,
        MaxReflectionAngle, SettleDuration, SettleTiltEase,
    },
    effect::{
        AnchorActive, AnchorPlanted,
        effects::{
            flash_step::FlashStepActive, size_boost::ActiveSizeBoosts,
            speed_boost::ActiveSpeedBoosts,
        },
    },
    shared::EntityScale,
};

/// Breaker entity data needed by bolt-breaker collision.
pub(crate) type CollisionQueryBreaker = (
    &'static Position2D,
    &'static BreakerTilt,
    &'static BreakerWidth,
    &'static BreakerHeight,
    &'static MaxReflectionAngle,
    Option<&'static ActiveSizeBoosts>,
    Option<&'static EntityScale>,
);

/// Breaker movement data — position, velocity, speed limits, and playfield clamping.
pub(crate) type MovementQuery = (
    &'static mut Position2D,
    &'static mut BreakerVelocity,
    &'static BreakerState,
    &'static MaxSpeed,
    &'static BreakerAcceleration,
    &'static BreakerDeceleration,
    &'static DecelEasing,
    &'static BreakerWidth,
    Option<&'static ActiveSpeedBoosts>,
    Option<&'static ActiveSizeBoosts>,
);

/// Breaker dash state machine data — full state, velocity, tilt, and all timing params.
///
/// Split into nested tuples to stay within Bevy's `QueryData` tuple element limit:
/// - Group 1: core dash state (mutable state + read-only config)
/// - Group 2: flash-step optional fields
pub(crate) type DashQuery = (
    (
        &'static mut BreakerState,
        &'static mut BreakerVelocity,
        &'static mut BreakerTilt,
        &'static mut BreakerStateTimer,
        &'static MaxSpeed,
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
    ),
    (
        Option<&'static FlashStepActive>,
        Option<&'static mut Position2D>,
        Option<&'static BreakerWidth>,
        Option<&'static ActiveSpeedBoosts>,
        Option<&'static ActiveSizeBoosts>,
    ),
);

/// Breaker reset data — mutable state cleared at node start.
pub(crate) type ResetQuery = (
    &'static mut Position2D,
    &'static mut BreakerState,
    &'static mut BreakerVelocity,
    &'static mut BreakerTilt,
    &'static mut BreakerStateTimer,
    &'static mut BumpState,
    &'static BreakerBaseY,
    Option<&'static mut PreviousPosition>,
);

/// Bump timing window data — state, timing/cooldown params.
pub(crate) type BumpTimingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpEarlyWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
    Option<&'static AnchorPlanted>,
    Option<&'static AnchorActive>,
);

/// Bump grading data — state, timing windows, and cooldowns for `grade_bump`.
pub(crate) type BumpGradingQuery = (
    &'static mut BumpState,
    &'static BumpPerfectWindow,
    &'static BumpLateWindow,
    &'static BumpPerfectCooldown,
    &'static BumpWeakCooldown,
    Option<&'static AnchorPlanted>,
    Option<&'static AnchorActive>,
);

/// Breaker data needed by the width boost visual system.
pub(crate) type WidthBoostVisualQuery = (
    &'static BreakerWidth,
    Option<&'static ActiveSizeBoosts>,
    &'static BreakerHeight,
    Option<&'static EntityScale>,
    &'static mut Scale2D,
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
