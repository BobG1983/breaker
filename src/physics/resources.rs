//! Physics domain resources.

use bevy::prelude::*;
use serde::Deserialize;

/// Configuration for physics mechanics.
#[derive(Resource, Debug, Clone)]
pub struct PhysicsConfig {
    /// Maximum reflection angle from vertical in radians.
    pub max_reflection_angle: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        PhysicsDefaults::default().into()
    }
}

/// Physics defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct PhysicsDefaults {
    /// Maximum reflection angle from vertical in radians.
    pub max_reflection_angle: f32,
}

impl Default for PhysicsDefaults {
    fn default() -> Self {
        Self {
            max_reflection_angle: 1.31,
        }
    }
}

impl From<PhysicsDefaults> for PhysicsConfig {
    fn from(d: PhysicsDefaults) -> Self {
        Self {
            max_reflection_angle: d.max_reflection_angle,
        }
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

    #[test]
    fn physics_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.physics.ron");
        let result: PhysicsDefaults = ron::de::from_str(ron_str).expect("physics RON should parse");
        assert!(result.max_reflection_angle > 0.0);
    }
}
