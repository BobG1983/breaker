//! Breaker domain components.

mod bolt_loss_behavior;
mod bump;
mod core;
mod dash;
mod movement;
mod previous_dash_state;
mod state;

pub use core::{
    Breaker, BreakerBaseY, BreakerInitialized, BreakerReflectionSpread, ExtraBreaker,
    PhantomBreaker, PrimaryBreaker,
};

pub use bolt_loss_behavior::BoltLossBehavior;
pub use bump::{
    BumpEarlyWindow, BumpFeedback, BumpFeedbackState, BumpLateWindow, BumpPerfectCooldown,
    BumpPerfectWindow, BumpState, BumpWeakCooldown,
};
pub use dash::{
    BrakeDecel, BrakeTilt, DashDuration, DashSpeedMultiplier, DashTilt, DashTiltEase,
    SettleDuration, SettleTiltEase,
};
pub use movement::{BreakerAcceleration, BreakerDeceleration, BreakerTilt, DecelEasing};
pub use previous_dash_state::PreviousDashState;
pub use state::{DashState, DashStateTimer};

pub use crate::shared::components::{BaseHeight, BaseWidth};
