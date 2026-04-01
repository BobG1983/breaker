//! Breaker movement components.

use bevy::{math::curve::easing::EaseFunction, prelude::*};

/// The breaker's current tilt angle in radians.
///
/// Positive = tilted right, negative = tilted left.
/// Affects bolt reflection angle on contact.
#[derive(Component, Debug, Default)]
pub struct BreakerTilt {
    /// Current tilt angle in radians.
    pub angle: f32,
    /// Start angle for the current ease animation.
    pub ease_start: f32,
    /// Target angle for the current ease animation.
    pub ease_target: f32,
}

/// Horizontal acceleration in world units per second squared.
#[derive(Component, Debug)]
pub struct BreakerAcceleration(pub f32);

/// Horizontal deceleration in world units per second squared.
#[derive(Component, Debug)]
pub struct BreakerDeceleration(pub f32);

/// Easing applied to deceleration based on speed ratio.
#[derive(Component, Debug)]
pub struct DecelEasing {
    /// Easing function for deceleration curve.
    pub ease: EaseFunction,
    /// Strength of eased deceleration (0.0 = constant decel, higher = more speed-dependent).
    pub strength: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_tilt_default_is_zero() {
        let tilt = BreakerTilt::default();
        assert!((tilt.angle).abs() < f32::EPSILON);
        assert!((tilt.ease_start).abs() < f32::EPSILON);
        assert!((tilt.ease_target).abs() < f32::EPSILON);
    }
}
