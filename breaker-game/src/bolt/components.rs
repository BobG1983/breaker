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
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            value: Vec2::new(x, y),
        }
    }

    /// Returns the current speed (magnitude of velocity).
    #[must_use]
    pub fn speed(&self) -> f32 {
        self.value.length()
    }

    /// Returns the normalized direction vector.
    #[must_use]
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

/// Base speed in world units per second.
#[derive(Component, Debug)]
pub struct BoltBaseSpeed(pub f32);

/// Minimum speed cap.
#[derive(Component, Debug)]
pub struct BoltMinSpeed(pub f32);

/// Maximum speed cap.
#[derive(Component, Debug)]
pub struct BoltMaxSpeed(pub f32);

/// Bolt radius in world units.
#[derive(Component, Debug)]
pub struct BoltRadius(pub f32);

/// Vertical offset above the breaker where the bolt spawns.
#[derive(Component, Debug)]
pub struct BoltSpawnOffsetY(pub f32);

/// Vertical offset above the breaker for bolt respawn after loss.
#[derive(Component, Debug)]
pub struct BoltRespawnOffsetY(pub f32);

/// Maximum respawn angle spread from vertical in radians.
#[derive(Component, Debug)]
pub struct BoltRespawnAngleSpread(pub f32);

/// Initial launch angle from vertical in radians.
#[derive(Component, Debug)]
pub struct BoltInitialAngle(pub f32);

/// Marker for extra bolts spawned by archetype consequences (e.g. Prism).
///
/// Extra bolts are despawned on loss rather than respawned. Only the
/// baseline bolt (without this marker) respawns.
#[derive(Component, Debug)]
pub struct ExtraBolt;

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
        use crate::breaker::resources::BreakerConfig;
        let mut vel = BoltVelocity::new(1.0, 5.0);
        let original = vel.value;
        vel.enforce_min_angle(
            BreakerConfig::default()
                .min_angle_from_horizontal
                .to_radians(),
        );
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

    #[test]
    fn enforce_min_angle_horizontal_defaults_upward() {
        use std::f32::consts::FRAC_PI_4;
        let mut vel = BoltVelocity::new(10.0, 0.0);
        let speed_before = vel.speed();
        vel.enforce_min_angle(FRAC_PI_4);
        let speed_after = vel.speed();
        assert!(
            (speed_before - speed_after).abs() < 1e-4,
            "speed should be preserved"
        );
        assert!(
            vel.value.y > 0.0,
            "horizontal velocity should default to upward"
        );
        let angle = vel.value.y.abs().atan2(vel.value.x.abs());
        assert!(
            angle >= FRAC_PI_4 - 1e-4,
            "angle should be at least min_angle"
        );
    }
}

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// enforce_min_angle never changes the speed magnitude.
        #[test]
        fn enforce_min_angle_preserves_speed(
            vx in -500.0_f32..500.0,
            vy in -500.0_f32..500.0,
            min_deg in 5.0_f32..45.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            let speed_before = vel.speed();
            if speed_before < f32::EPSILON {
                return Ok(());
            }
            vel.enforce_min_angle(min_deg.to_radians());
            let speed_after = vel.speed();
            prop_assert!(
                (speed_before - speed_after).abs() < 0.1,
                "speed should be preserved: {speed_before} vs {speed_after}"
            );
        }

        /// enforce_min_angle never produces NaN or infinity.
        #[test]
        fn enforce_min_angle_never_nan(
            vx in -1000.0_f32..1000.0,
            vy in -1000.0_f32..1000.0,
            min_deg in 1.0_f32..89.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            vel.enforce_min_angle(min_deg.to_radians());
            prop_assert!(vel.value.x.is_finite(), "x should be finite: {}", vel.value.x);
            prop_assert!(vel.value.y.is_finite(), "y should be finite: {}", vel.value.y);
        }

        /// After enforce_min_angle, the angle from horizontal is >= min_angle.
        #[test]
        fn enforce_min_angle_result_meets_minimum(
            vx in -500.0_f32..500.0,
            vy in -500.0_f32..500.0,
            min_deg in 5.0_f32..45.0,
        ) {
            let mut vel = BoltVelocity::new(vx, vy);
            if vel.speed() < f32::EPSILON {
                return Ok(());
            }
            let min_rad = min_deg.to_radians();
            vel.enforce_min_angle(min_rad);
            let angle = vel.value.y.abs().atan2(vel.value.x.abs());
            prop_assert!(
                angle >= min_rad - 1e-4,
                "angle {angle:.4} should be >= min {min_rad:.4}, vel=({}, {})",
                vel.value.x, vel.value.y
            );
        }
    }
}
