//! Bolt domain resources.

use bevy::prelude::*;

/// Configuration for bolt mechanics.
#[derive(Resource, Debug, Clone)]
pub struct BoltConfig {
    /// Base speed in world units per second.
    pub base_speed: f32,
    /// Minimum speed cap.
    pub min_speed: f32,
    /// Maximum speed cap.
    pub max_speed: f32,
    /// Minimum angle from horizontal in radians.
    pub min_angle_from_horizontal: f32,
    /// Bolt radius in world units.
    pub radius: f32,
    /// Vertical offset above the breaker where the bolt spawns.
    pub spawn_offset_y: f32,
    /// Initial launch angle from vertical in radians.
    pub initial_angle: f32,
    /// Vertical offset above the breaker for bolt respawn after loss.
    pub respawn_offset_y: f32,
    /// RGB values for the bolt HDR color.
    pub color_rgb: [f32; 3],
}

impl Default for BoltConfig {
    fn default() -> Self {
        crate::screen::defaults::BoltDefaults::default().into()
    }
}

impl BoltConfig {
    /// Bolt color as a Bevy [`Color`].
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
    fn default_speed_within_bounds() {
        let config = BoltConfig::default();
        assert!(config.base_speed >= config.min_speed);
        assert!(config.base_speed <= config.max_speed);
    }

    #[test]
    fn min_angle_is_positive() {
        let config = BoltConfig::default();
        assert!(config.min_angle_from_horizontal > 0.0);
    }
}
