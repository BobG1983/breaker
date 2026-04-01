//! Breaker domain components.

mod bump;
mod core;
mod dash;
mod movement;
mod state;

pub use core::{
    BaseHeight, BaseWidth, Breaker, BreakerBaseY, BreakerInitialized, BreakerReflectionSpread,
};

pub use bump::{
    BumpEarlyWindow, BumpFeedback, BumpFeedbackState, BumpLateWindow, BumpPerfectCooldown,
    BumpPerfectWindow, BumpState, BumpWeakCooldown,
};
pub use dash::{
    BrakeDecel, BrakeTilt, DashDuration, DashSpeedMultiplier, DashTilt, DashTiltEase,
    SettleDuration, SettleTiltEase,
};
pub use movement::{BreakerAcceleration, BreakerDeceleration, BreakerTilt, DecelEasing};
pub use state::{DashState, DashStateTimer};
