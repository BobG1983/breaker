//! Bolt domain resources.

use bevy::prelude::*;
use breaker_derive::GameConfig;
use serde::Deserialize;

/// Bolt defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "BoltConfig")]
pub struct BoltDefaults {
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
    /// RGB values for the bolt HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BoltDefaults {
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
            color_rgb: [6.0, 5.0, 0.5],
        }
    }
}

impl BoltConfig {
    /// Bolt color as a Bevy [`Color`].
    #[must_use]
    pub fn color(&self) -> Color {
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
}
