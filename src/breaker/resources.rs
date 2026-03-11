//! Breaker domain resources.

use bevy::prelude::*;

/// Configuration for breaker mechanics.
///
/// All tunable breaker parameters in one place. Loaded as a `Resource`.
#[derive(Resource, Debug, Clone)]
pub struct BreakerConfig {
    /// Half-width of the breaker in world units.
    pub half_width: f32,
    /// Half-height of the breaker in world units.
    pub half_height: f32,
    /// Maximum horizontal speed in world units per second.
    pub max_speed: f32,
    /// Horizontal acceleration in world units per second squared.
    pub acceleration: f32,
    /// Horizontal deceleration (friction) in world units per second squared.
    pub deceleration: f32,
    /// Dash speed multiplier relative to max speed.
    pub dash_speed_multiplier: f32,
    /// Duration of the dash in seconds.
    pub dash_duration: f32,
    /// Brake deceleration multiplier relative to normal deceleration.
    pub brake_decel_multiplier: f32,
    /// Duration of the settle phase in seconds.
    pub settle_duration: f32,
    /// Maximum tilt angle during dash in radians.
    pub dash_tilt_angle: f32,
    /// Maximum tilt angle during brake in radians.
    pub brake_tilt_angle: f32,
    /// Y position of the breaker.
    pub y_position: f32,
    /// Duration of the bump active window in seconds.
    pub bump_duration: f32,
    /// Cooldown between bumps in seconds.
    pub bump_cooldown: f32,
    /// Perfect bump timing window (seconds).
    pub perfect_bump_window: f32,
    /// Early bump window (seconds).
    pub early_bump_window: f32,
    /// Velocity multiplier for perfect bump.
    pub perfect_bump_multiplier: f32,
    /// Velocity multiplier for early/late bump.
    pub weak_bump_multiplier: f32,
    /// Velocity multiplier for no bump.
    pub no_bump_multiplier: f32,
    /// RGB values for the breaker HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BreakerConfig {
    fn default() -> Self {
        crate::screen::defaults::BreakerDefaults::default().into()
    }
}

impl BreakerConfig {
    /// Breaker color as a Bevy [`Color`].
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn color(&self) -> Color {
        Color::srgb(self.color_rgb[0], self.color_rgb[1], self.color_rgb[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_positive_dimensions() {
        let config = BreakerConfig::default();
        assert!(config.half_width > 0.0);
        assert!(config.half_height > 0.0);
    }

    #[test]
    fn default_config_has_positive_speeds() {
        let config = BreakerConfig::default();
        assert!(config.max_speed > 0.0);
        assert!(config.acceleration > 0.0);
        assert!(config.deceleration > 0.0);
    }

    #[test]
    fn dash_speed_exceeds_normal() {
        let config = BreakerConfig::default();
        assert!(config.dash_speed_multiplier > 1.0);
    }

    #[test]
    fn perfect_bump_multiplier_exceeds_weak() {
        let config = BreakerConfig::default();
        assert!(config.perfect_bump_multiplier > config.weak_bump_multiplier);
    }
}
