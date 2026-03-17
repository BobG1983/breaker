//! Bump system components.

use bevy::{math::curve::easing::EaseFunction, prelude::*};

/// Tracks the bump state for timing-grade calculations.
#[derive(Component, Debug)]
pub struct BumpState {
    /// Whether a forward bump window is open (pressed, waiting for bolt).
    pub active: bool,
    /// Countdown from (`early_window` + `perfect_window`) — forward window.
    pub timer: f32,
    /// Countdown from (`perfect_window` + `late_window`) after bolt hit — retroactive window.
    pub post_hit_timer: f32,
    /// Cooldown remaining before another bump can be triggered (seconds).
    pub cooldown: f32,
}

impl Default for BumpState {
    fn default() -> Self {
        Self {
            active: false,
            timer: 0.0,
            post_hit_timer: 0.0,
            cooldown: 0.0,
        }
    }
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

/// Perfect bump timing window (seconds, each side of T=0).
#[derive(Component, Debug)]
pub struct BumpPerfectWindow(pub f32);

/// Early bump window (seconds, before perfect zone).
#[derive(Component, Debug)]
pub struct BumpEarlyWindow(pub f32);

/// Late bump window (seconds, after perfect zone).
#[derive(Component, Debug)]
pub struct BumpLateWindow(pub f32);

/// Cooldown after a perfect bump in seconds.
#[derive(Component, Debug)]
pub struct BumpPerfectCooldown(pub f32);

/// Cooldown after an early/late bump or whiff in seconds.
#[derive(Component, Debug)]
pub struct BumpWeakCooldown(pub f32);

/// Velocity multiplier for perfect bump.
#[derive(Component, Debug)]
pub struct BumpPerfectMultiplier(pub f32);

/// Velocity multiplier for early/late bump.
#[derive(Component, Debug)]
pub struct BumpWeakMultiplier(pub f32);

/// Parameters for the bump pop visual animation.
#[derive(Component, Debug, Clone)]
pub struct BumpVisualParams {
    /// Total duration of the animation (seconds).
    pub duration: f32,
    /// Maximum Y offset at peak (world units).
    pub peak: f32,
    /// Fraction of duration spent rising (0.0–1.0).
    pub peak_fraction: f32,
    /// Easing for the rise phase.
    pub rise_ease: EaseFunction,
    /// Easing for the fall phase.
    pub fall_ease: EaseFunction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_state_default_is_inactive() {
        let bump = BumpState::default();
        assert!(!bump.active);
        assert!((bump.timer - 0.0).abs() < f32::EPSILON);
        assert!((bump.cooldown - 0.0).abs() < f32::EPSILON);
    }
}
