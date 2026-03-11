//! Breaker domain resources.

use bevy::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

/// Breaker defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "BreakerConfig")]
pub struct BreakerDefaults {
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
    /// Duration of the bump pop animation in seconds.
    pub bump_visual_duration: f32,
    /// Maximum Y offset at the peak of the bump pop animation (world units).
    pub bump_visual_peak: f32,
}

impl Default for BreakerDefaults {
    fn default() -> Self {
        Self {
            half_width: 60.0,
            half_height: 10.0,
            max_speed: 500.0,
            acceleration: 3000.0,
            deceleration: 2500.0,
            dash_speed_multiplier: 2.0,
            dash_duration: 0.15,
            brake_decel_multiplier: 4.0,
            settle_duration: 0.12,
            dash_tilt_angle: 0.26,
            brake_tilt_angle: 0.44,
            y_position: -250.0,
            bump_duration: 0.3,
            bump_cooldown: 0.3,
            perfect_bump_window: 0.05,
            early_bump_window: 0.15,
            perfect_bump_multiplier: 1.5,
            weak_bump_multiplier: 0.8,
            no_bump_multiplier: 1.0,
            color_rgb: [0.2, 2.0, 3.0],
            bump_visual_duration: 0.15,
            bump_visual_peak: 6.0,
        }
    }
}

impl BreakerConfig {
    /// Breaker color as a Bevy [`Color`].
    #[must_use]
    pub fn color(&self) -> Color {
        crate::shared::color_from_rgb(self.color_rgb)
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

    #[test]
    fn breaker_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.breaker.ron");
        let result: BreakerDefaults = ron::de::from_str(ron_str).expect("breaker RON should parse");
        assert!(result.half_width > 0.0);
    }
}
