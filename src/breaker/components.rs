//! Breaker domain components.

use bevy::prelude::*;

/// Marker component identifying the breaker entity.
#[derive(Component, Debug)]
pub struct Breaker;

/// The breaker's current horizontal velocity in world units per second.
#[derive(Component, Debug, Default)]
pub struct BreakerVelocity {
    /// Horizontal velocity. Positive = right, negative = left.
    pub x: f32,
}

/// The breaker's movement state machine.
///
/// Transitions: Idle → Dashing → Braking → Settling → Idle.
/// Normal movement is available in Idle and Settling states.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Neutral state — normal horizontal movement available.
    #[default]
    Idle,
    /// Burst horizontal speed — breaker tilts in movement direction.
    Dashing,
    /// Rapid deceleration — breaker tilts opposite to movement direction.
    Braking,
    /// Returning to neutral — tilt returns to flat.
    Settling,
}

/// The breaker's current tilt angle in radians.
///
/// Positive = tilted right, negative = tilted left.
/// Affects bolt reflection angle on contact.
#[derive(Component, Debug, Default)]
pub struct BreakerTilt {
    /// Current tilt angle in radians.
    pub angle: f32,
    /// Angle captured when entering Settling, used for frame-rate-independent lerp.
    pub settle_start_angle: f32,
}

/// Tracks the bump state for timing-grade calculations.
#[derive(Component, Debug)]
pub struct BumpState {
    /// Whether a bump is currently active.
    pub active: bool,
    /// Time remaining in the bump window (seconds).
    pub timer: f32,
    /// Cooldown remaining before another bump can be triggered (seconds).
    pub cooldown: f32,
}

/// Tracks the bump pop animation — an eased upward offset on the breaker.
#[derive(Component, Debug)]
pub struct BumpVisual {
    /// Time remaining in the animation (seconds).
    pub timer: f32,
    /// Total duration of the animation (seconds).
    pub duration: f32,
    /// Maximum Y offset at peak (world units).
    pub peak_offset: f32,
}

/// Tracks the remaining time in timed breaker states (Dashing, Settling).
#[derive(Component, Debug, Default)]
pub struct BreakerStateTimer {
    /// Remaining time in the current timed state.
    pub remaining: f32,
}

impl Default for BumpState {
    fn default() -> Self {
        Self {
            active: false,
            timer: 0.0,
            cooldown: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_velocity_default_is_zero() {
        let vel = BreakerVelocity::default();
        assert!((vel.x - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_state_default_is_idle() {
        assert_eq!(BreakerState::default(), BreakerState::Idle);
    }

    #[test]
    fn breaker_tilt_default_is_zero() {
        let tilt = BreakerTilt::default();
        assert!((tilt.angle - 0.0).abs() < f32::EPSILON);
        assert!((tilt.settle_start_angle - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bump_state_default_is_inactive() {
        let bump = BumpState::default();
        assert!(!bump.active);
        assert!((bump.timer - 0.0).abs() < f32::EPSILON);
        assert!((bump.cooldown - 0.0).abs() < f32::EPSILON);
    }
}
