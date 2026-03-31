//! Bolt domain resources.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;

/// Base damage dealt by a bolt hit. Fixed game-design constant.
pub const BASE_BOLT_DAMAGE: f32 = 10.0;

/// Bolt configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "BoltDefaults",
    path = "config/defaults.bolt.ron",
    ext = "bolt.ron"
)]
pub struct BoltConfig {
    /// Base speed in world units per second.
    pub base_speed: f32,
    /// Minimum speed cap.
    pub min_speed: f32,
    /// Maximum speed cap.
    pub max_speed: f32,
    /// Bolt radius in world units.
    pub radius: f32,
    /// Vertical offset above the breaker where the bolt spawns.
    pub spawn_offset_y: f32,
    /// Initial launch angle from vertical in radians.
    pub initial_angle: f32,
    /// Vertical offset above the breaker for bolt respawn after loss.
    pub respawn_offset_y: f32,
    /// Maximum respawn angle spread from vertical in radians.
    ///
    /// On bolt loss, the respawn direction is randomized within
    /// `[-respawn_angle_spread, +respawn_angle_spread]` from straight up.
    pub respawn_angle_spread: f32,
    /// Minimum angle from horizontal in degrees. Prevents the bolt from
    /// traveling too close to horizontal.
    pub min_angle_horizontal: f32,
    /// Minimum angle from vertical in degrees. Prevents the bolt from
    /// traveling too close to vertical.
    pub min_angle_vertical: f32,
    /// RGB values for the bolt HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BoltConfig {
    fn default() -> Self {
        Self {
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            spawn_offset_y: 30.0,
            initial_angle: 0.26,
            respawn_offset_y: 30.0,
            respawn_angle_spread: 0.524, // ~30 degrees
            min_angle_horizontal: 5.0,   // degrees
            min_angle_vertical: 5.0,     // degrees
            color_rgb: [6.0, 5.0, 0.5],
        }
    }
}

impl BoltConfig {
    /// Bolt color as a Bevy [`Color`].
    #[must_use]
    pub const fn color(&self) -> Color {
        crate::shared::color_from_rgb(self.color_rgb)
    }

    /// Initial launch velocity based on `base_speed` and `initial_angle`.
    #[must_use]
    pub fn initial_velocity(&self) -> Vec2 {
        Vec2::new(
            self.base_speed * self.initial_angle.sin(),
            self.base_speed * self.initial_angle.cos(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_speed_within_bounds() {
        let config = BoltConfig::default();
        assert!(config.base_speed >= config.min_speed);
        assert!(config.base_speed <= config.max_speed);
    }

    #[test]
    fn bolt_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.bolt.ron");
        let result: BoltDefaults = ron::de::from_str(ron_str).expect("bolt RON should parse");
        assert!(result.base_speed > 0.0);
    }

    #[test]
    fn base_bolt_damage_equals_10() {
        assert!((BASE_BOLT_DAMAGE - 10.0_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn initial_velocity_trig_math() {
        let config = BoltConfig::default();
        // default: base_speed = 400.0, initial_angle = 0.26
        let v = config.initial_velocity();

        let expected_x = 400.0 * 0.26_f32.sin();
        let expected_y = 400.0 * 0.26_f32.cos();

        assert!(
            (v.x - expected_x).abs() < 1e-2,
            "x should be ~{expected_x}, got {}",
            v.x
        );
        assert!(
            (v.y - expected_y).abs() < 1e-2,
            "y should be ~{expected_y}, got {}",
            v.y
        );

        // Edge case: initial_angle = 0.0 -> straight up
        let mut config_zero = config.clone();
        config_zero.initial_angle = 0.0;
        let v_zero = config_zero.initial_velocity();
        assert!(
            v_zero.x.abs() < 1e-6,
            "angle 0.0 should give x=0.0, got {}",
            v_zero.x
        );
        assert!(
            (v_zero.y - config_zero.base_speed).abs() < 1e-2,
            "angle 0.0 should give y=base_speed ({}), got {}",
            config_zero.base_speed,
            v_zero.y
        );

        // Edge case: initial_angle = PI/2 -> horizontal
        let mut config_half_pi = config;
        config_half_pi.initial_angle = std::f32::consts::FRAC_PI_2;
        let v_half = config_half_pi.initial_velocity();
        assert!(
            (v_half.x - config_half_pi.base_speed).abs() < 1e-2,
            "angle PI/2 should give x=base_speed ({}), got {}",
            config_half_pi.base_speed,
            v_half.x
        );
        assert!(
            v_half.y.abs() < 1e-2,
            "angle PI/2 should give y~0.0, got {}",
            v_half.y
        );
    }
}
