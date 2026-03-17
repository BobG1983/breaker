//! Interpolation domain components.

use bevy::prelude::*;

/// Marker component — entities with this get visual interpolation between
/// fixed-timestep ticks.
#[derive(Component, Debug)]
pub struct InterpolateTransform;

/// Stores the authoritative physics positions from the last two `FixedUpdate` ticks.
///
/// The rendering system lerps between `previous` and `current` using
/// `Time<Fixed>::overstep_fraction()` as the alpha value.
#[derive(Component, Debug, Clone)]
pub struct PhysicsTranslation {
    /// Position at the start of the most recent `FixedUpdate` tick.
    pub previous: Vec3,
    /// Position at the end of the most recent `FixedUpdate` tick.
    pub current: Vec3,
}

impl PhysicsTranslation {
    /// Creates a new physics translation with both previous and current at
    /// the same position (no interpolation on first frame).
    #[must_use]
    pub const fn new(position: Vec3) -> Self {
        Self {
            previous: position,
            current: position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_both_to_same_position() {
        let pos = Vec3::new(10.0, 20.0, 1.0);
        let pt = PhysicsTranslation::new(pos);
        assert_eq!(pt.previous, pos);
        assert_eq!(pt.current, pos);
    }
}
