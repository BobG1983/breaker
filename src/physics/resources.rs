//! Physics domain resources.

use bevy::prelude::*;

/// Configuration for physics mechanics.
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    /// Maximum reflection angle from vertical in radians.
    pub max_reflection_angle: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        crate::screen::defaults::PhysicsDefaults::default().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_max_reflection_angle_positive() {
        let config = PhysicsConfig::default();
        assert!(config.max_reflection_angle > 0.0);
    }
}
