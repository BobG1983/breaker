//! Breaker domain `QueryData` structs — named-field query types.

use bevy::{ecs::query::QueryData, prelude::*};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, Scale2D, Velocity2D,
};

use crate::{
    breaker::components::{
        BaseHeight, BaseWidth, BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY,
        BreakerDeceleration, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow, BumpLateWindow,
        BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown, DashDuration,
        DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase, DecelEasing,
        SettleDuration, SettleTiltEase,
    },
    prelude::{components::FlashStepActive, *},
};

// ── QueryData structs ───────────────────────────────────────────────────

/// Breaker collision data for bolt-breaker collision (read-only).
#[derive(QueryData)]
pub(crate) struct BreakerCollisionData {
    /// World position.
    pub position: &'static Position2D,
    /// Current tilt angle.
    pub tilt: &'static BreakerTilt,
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Maximum reflection angle.
    pub reflection_spread: &'static BreakerReflectionSpread,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
}

/// Breaker entity data for cell/wall collision (read-only).
#[derive(QueryData)]
pub(crate) struct BreakerSizeData {
    /// The breaker entity.
    pub entity: Entity,
    /// World position.
    pub position: &'static Position2D,
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker movement data — mutable position/velocity, read-only config.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerMovementData {
    /// Mutable world position.
    pub position: &'static mut Position2D,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Current dash state (read-only).
    pub state: &'static DashState,
    /// Maximum movement speed.
    pub max_speed: &'static MaxSpeed,
    /// Input acceleration rate.
    pub acceleration: &'static BreakerAcceleration,
    /// Deceleration rate.
    pub deceleration: &'static BreakerDeceleration,
    /// Deceleration easing parameters.
    pub decel_easing: &'static DecelEasing,
    /// Base width for playfield clamping.
    pub base_width: &'static BaseWidth,
    /// Active speed boost multipliers.
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker dash state machine data — full state, velocity, tilt, and all timing params.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerDashData {
    /// Mutable dash state.
    pub state: &'static mut DashState,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Mutable tilt.
    pub tilt: &'static mut BreakerTilt,
    /// Mutable dash state timer.
    pub timer: &'static mut DashStateTimer,
    /// Maximum movement speed.
    pub max_speed: &'static MaxSpeed,
    /// Deceleration rate.
    pub deceleration: &'static BreakerDeceleration,
    /// Deceleration easing parameters.
    pub decel_easing: &'static DecelEasing,
    /// Dash speed multiplier.
    pub dash_speed: &'static DashSpeedMultiplier,
    /// Dash duration in seconds.
    pub dash_duration: &'static DashDuration,
    /// Dash tilt angle.
    pub dash_tilt: &'static DashTilt,
    /// Dash tilt easing function.
    pub dash_tilt_ease: &'static DashTiltEase,
    /// Brake tilt configuration.
    pub brake_tilt: &'static BrakeTilt,
    /// Brake deceleration multiplier.
    pub brake_decel: &'static BrakeDecel,
    /// Settle duration in seconds.
    pub settle_duration: &'static SettleDuration,
    /// Settle tilt easing function.
    pub settle_tilt_ease: &'static SettleTiltEase,
    /// Flash step active marker.
    pub flash_step: Option<&'static FlashStepActive>,
    /// Mutable position (optional — for flash step teleport).
    pub position: Option<&'static mut Position2D>,
    /// Base width (optional — for flash step playfield clamping).
    pub base_width: Option<&'static BaseWidth>,
    /// Active speed boost multipliers.
    pub speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
}

/// Breaker reset data — mutable state cleared at node start.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerResetData {
    /// Mutable world position.
    pub position: &'static mut Position2D,
    /// Mutable dash state.
    pub state: &'static mut DashState,
    /// Mutable velocity.
    pub velocity: &'static mut Velocity2D,
    /// Mutable tilt.
    pub tilt: &'static mut BreakerTilt,
    /// Mutable dash state timer.
    pub timer: &'static mut DashStateTimer,
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Base Y position (read-only).
    pub base_y: &'static BreakerBaseY,
    /// Previous position snapshot (optional, mutable).
    pub prev_position: Option<&'static mut PreviousPosition>,
}

/// Bump timing window data — state, timing/cooldown params.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerBumpTimingData {
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Early bump window duration.
    pub early_window: &'static BumpEarlyWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
    /// Perfect bump cooldown duration.
    pub perfect_cooldown: &'static BumpPerfectCooldown,
    /// Weak bump cooldown duration.
    pub weak_cooldown: &'static BumpWeakCooldown,
    /// Anchor planted marker.
    pub anchor_planted: Option<&'static AnchorPlanted>,
    /// Anchor active configuration.
    pub anchor_active: Option<&'static AnchorActive>,
}

/// Bump grading data — state, timing windows, and cooldowns for `grade_bump`.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BreakerBumpGradingData {
    /// Mutable bump state.
    pub bump: &'static mut BumpState,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
    /// Perfect bump cooldown duration.
    pub perfect_cooldown: &'static BumpPerfectCooldown,
    /// Weak bump cooldown duration.
    pub weak_cooldown: &'static BumpWeakCooldown,
    /// Anchor planted marker.
    pub anchor_planted: Option<&'static AnchorPlanted>,
    /// Anchor active configuration.
    pub anchor_active: Option<&'static AnchorActive>,
}

/// Breaker data for the `sync_breaker_scale` system.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct SyncBreakerScaleData {
    /// Base width in world units.
    pub base_width: &'static BaseWidth,
    /// Base height in world units.
    pub base_height: &'static BaseHeight,
    /// Mutable scale for rendering.
    pub scale: &'static mut Scale2D,
    /// Active size boost multipliers.
    pub size_boosts: Option<&'static ActiveSizeBoosts>,
    /// Node scaling factor.
    pub node_scale: Option<&'static NodeScalingFactor>,
    /// Minimum width constraint.
    pub min_w: Option<&'static crate::shared::size::MinWidth>,
    /// Maximum width constraint.
    pub max_w: Option<&'static crate::shared::size::MaxWidth>,
    /// Minimum height constraint.
    pub min_h: Option<&'static crate::shared::size::MinHeight>,
    /// Maximum height constraint.
    pub max_h: Option<&'static crate::shared::size::MaxHeight>,
}

/// Breaker bump telemetry — state, bump, tilt, velocity, and window sizes.
#[cfg(feature = "dev")]
#[derive(QueryData)]
pub(crate) struct BreakerTelemetryData {
    /// Current dash state.
    pub state: &'static DashState,
    /// Bump state.
    pub bump: &'static BumpState,
    /// Current tilt.
    pub tilt: &'static BreakerTilt,
    /// Current velocity.
    pub velocity: &'static Velocity2D,
    /// Perfect bump window duration.
    pub perfect_window: &'static BumpPerfectWindow,
    /// Early bump window duration.
    pub early_window: &'static BumpEarlyWindow,
    /// Late bump window duration.
    pub late_window: &'static BumpLateWindow,
}
