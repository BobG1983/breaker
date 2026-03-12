//! Breaker domain resources.

use bevy::{math::curve::easing::EaseFunction, prelude::*};
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
    /// Cooldown after a perfect bump in seconds.
    pub perfect_bump_cooldown: f32,
    /// Cooldown after an early/late bump or whiff in seconds.
    pub weak_bump_cooldown: f32,
    /// Perfect bump timing window (seconds, each side of T=0).
    pub perfect_window: f32,
    /// Early bump window (seconds, before perfect zone).
    pub early_window: f32,
    /// Late bump window (seconds, after perfect zone).
    pub late_window: f32,
    /// Velocity multiplier for perfect bump.
    pub perfect_bump_multiplier: f32,
    /// Velocity multiplier for early/late bump.
    pub weak_bump_multiplier: f32,
    /// RGB values for the breaker HDR color.
    pub color_rgb: [f32; 3],
    /// Duration of the bump pop animation in seconds.
    pub bump_visual_duration: f32,
    /// Maximum Y offset at the peak of the bump pop animation (world units).
    pub bump_visual_peak: f32,
    /// Fraction of bump pop duration spent rising (0.0–1.0).
    pub bump_visual_peak_fraction: f32,
    /// Easing for the rise phase of the bump pop.
    pub bump_visual_rise_ease: EaseFunction,
    /// Easing for the fall phase of the bump pop.
    pub bump_visual_fall_ease: EaseFunction,
    /// Easing for settle tilt return to zero.
    pub settle_tilt_ease: EaseFunction,
    /// Easing applied to deceleration based on speed ratio.
    pub decel_ease: EaseFunction,
    /// Strength of eased deceleration (0.0 = constant decel, higher = more speed-dependent).
    pub decel_ease_strength: f32,
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
            perfect_bump_cooldown: 0.0,
            weak_bump_cooldown: 0.15,
            perfect_window: 0.05,
            early_window: 0.15,
            late_window: 0.15,
            perfect_bump_multiplier: 1.5,
            weak_bump_multiplier: 0.8,
            color_rgb: [0.2, 2.0, 3.0],
            bump_visual_duration: 0.15,
            bump_visual_peak: 12.0,
            bump_visual_peak_fraction: 0.3,
            bump_visual_rise_ease: EaseFunction::CubicOut,
            bump_visual_fall_ease: EaseFunction::QuadraticIn,
            settle_tilt_ease: EaseFunction::CubicOut,
            decel_ease: EaseFunction::QuadraticIn,
            decel_ease_strength: 1.0,
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
