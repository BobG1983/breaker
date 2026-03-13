//! Dash, brake, and settle components.

use bevy::{math::curve::easing::EaseFunction, prelude::*};

/// Dash speed multiplier relative to max speed.
#[derive(Component, Debug)]
pub struct DashSpeedMultiplier(pub f32);

/// Duration of the dash in seconds.
#[derive(Component, Debug)]
pub struct DashDuration(pub f32);

/// Maximum tilt angle during dash in radians.
#[derive(Component, Debug)]
pub struct DashTilt(pub f32);

/// Brake tilt configuration — angle, duration, and easing.
#[derive(Component, Debug)]
pub struct BrakeTilt {
    /// Maximum tilt angle during brake in radians.
    pub angle: f32,
    /// Duration of the brake tilt ease in seconds.
    pub duration: f32,
    /// Easing function for the brake tilt.
    pub ease: EaseFunction,
}

/// Brake deceleration multiplier relative to normal deceleration.
#[derive(Component, Debug)]
pub struct BrakeDecel(pub f32);

/// Duration of the settle phase in seconds.
#[derive(Component, Debug)]
pub struct SettleDuration(pub f32);

/// Easing for dash tilt ramp-up (fast in, slower settle).
#[derive(Component, Debug)]
pub struct DashTiltEase(pub EaseFunction);

/// Easing for settle tilt return to zero.
#[derive(Component, Debug)]
pub struct SettleTiltEase(pub EaseFunction);
