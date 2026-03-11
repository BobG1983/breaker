//! Bolt domain components.

use bevy::prelude::*;

/// Marker component identifying the bolt entity.
#[derive(Component, Debug)]
pub struct Bolt;

/// Marker component indicating the bolt is hovering above the breaker,
/// waiting for the player to launch it. Present only on the first node.
#[derive(Component, Debug)]
pub struct BoltServing;

/// The bolt's velocity in world units per second.
#[derive(Component, Debug, Clone)]
pub struct BoltVelocity {
    /// Velocity vector (x, y).
    pub value: Vec2,
}

impl BoltVelocity {
    /// Creates a new bolt velocity.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            value: Vec2::new(x, y),
        }
    }

    /// Returns the current speed (magnitude of velocity).
    pub fn speed(&self) -> f32 {
        self.value.length()
    }

    /// Returns the normalized direction vector.
    pub fn direction(&self) -> Vec2 {
        self.value.normalize_or_zero()
    }

    /// Adjusts velocity so it never gets too close to horizontal.
    ///
    /// If the angle from horizontal is less than `min_angle`, rotates the
    /// vector to the minimum angle while preserving speed and Y sign.
    pub fn enforce_min_angle(&mut self, min_angle: f32) {
        let speed = self.value.length();
        if speed < f32::EPSILON {
            return;
        }

        let angle_from_horizontal = self.value.y.abs().atan2(self.value.x.abs());
        if angle_from_horizontal < min_angle {
            let sign_x = self.value.x.signum();
            let sign_y = if self.value.y.abs() < f32::EPSILON {
                1.0 // Default to upward if perfectly horizontal
            } else {
                self.value.y.signum()
            };
            self.value.x = sign_x * speed * min_angle.cos();
            self.value.y = sign_y * speed * min_angle.sin();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bolt_velocity_speed() {
        let vel = BoltVelocity::new(3.0, 4.0);
        assert!((vel.speed() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bolt_velocity_direction_normalized() {
        let vel = BoltVelocity::new(3.0, 4.0);
        let dir = vel.direction();
        assert!((dir.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bolt_velocity_zero_direction() {
        let vel = BoltVelocity::new(0.0, 0.0);
        assert_eq!(vel.direction(), Vec2::ZERO);
    }

    #[test]
    fn enforce_min_angle_leaves_steep_unchanged() {
        use crate::bolt::resources::BoltConfig;
        let mut vel = BoltVelocity::new(1.0, 5.0);
        let original = vel.value;
        vel.enforce_min_angle(BoltConfig::default().min_angle_from_horizontal);
        assert!((vel.value.x - original.x).abs() < 1e-6);
        assert!((vel.value.y - original.y).abs() < 1e-6);
    }

    #[test]
    fn enforce_min_angle_corrects_shallow() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(10.0, 0.01);
        let speed_before = vel.speed();
        vel.enforce_min_angle(FRAC_PI_4);
        let speed_after = vel.speed();
        assert!((speed_before - speed_after).abs() < 1e-4);
        let angle = vel.value.y.abs().atan2(vel.value.x.abs());
        assert!(angle >= FRAC_PI_4 - 1e-4);
    }

    #[test]
    fn enforce_min_angle_preserves_signs() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(-10.0, -0.01);
        vel.enforce_min_angle(FRAC_PI_4);
        assert!(vel.value.x < 0.0);
        assert!(vel.value.y < 0.0);
    }
}
