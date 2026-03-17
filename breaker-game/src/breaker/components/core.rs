//! Breaker core components.

use bevy::prelude::*;

/// Marker component identifying the breaker entity.
#[derive(Component, Debug)]
pub struct Breaker;

/// Full width of the breaker in world units.
#[derive(Component, Debug)]
pub struct BreakerWidth(pub f32);

impl BreakerWidth {
    /// Returns half the breaker width.
    #[must_use]
    pub fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Full height of the breaker in world units.
#[derive(Component, Debug)]
pub struct BreakerHeight(pub f32);

impl BreakerHeight {
    /// Returns half the breaker height.
    #[must_use]
    pub fn half_height(&self) -> f32 {
        self.0 / 2.0
    }
}

/// Y position of the breaker at rest.
#[derive(Component, Debug)]
pub struct BreakerBaseY(pub f32);

/// Maximum reflection angle from vertical in radians.
#[derive(Component, Debug)]
pub struct MaxReflectionAngle(pub f32);

/// Minimum angle from horizontal in radians.
#[derive(Component, Debug)]
pub struct MinAngleFromHorizontal(pub f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_width_half_width() {
        let w = BreakerWidth(120.0);
        assert!((w.half_width() - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn breaker_height_half_height() {
        let h = BreakerHeight(20.0);
        assert!((h.half_height() - 10.0).abs() < f32::EPSILON);
    }
}
