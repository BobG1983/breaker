//! Breaker domain components.

mod bump;
mod core;
mod dash;
mod movement;
mod state;

pub use core::{
    Breaker, BreakerBaseY, BreakerHeight, BreakerWidth, MaxReflectionAngle, MinAngleFromHorizontal,
};

pub use bump::{
    BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectMultiplier, BumpPerfectWindow,
    BumpState, BumpVisual, BumpVisualParams, BumpWeakCooldown, BumpWeakMultiplier,
};
pub use dash::{
    BrakeDecel, BrakeTilt, DashDuration, DashSpeedMultiplier, DashTilt, DashTiltEase,
    SettleDuration, SettleTiltEase,
};
pub use movement::{
    BreakerAcceleration, BreakerDeceleration, BreakerMaxSpeed, BreakerTilt, BreakerVelocity,
    DecelEasing,
};
pub use state::{BreakerState, BreakerStateTimer};
