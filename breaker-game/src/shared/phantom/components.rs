//! Shared components for phantom-breaker entities.

use bevy::prelude::*;

/// Remaining lifetime in seconds before the entity is despawned.
#[derive(Component, Debug, Clone, Copy)]
pub struct Lifespan {
    /// Remaining lifetime in seconds.
    pub remaining: f32,
}

/// Drives alpha oscillation on a `MeshMaterial2d<ColorMaterial>` entity.
///
/// Alpha is derived from global elapsed time only — no per-entity accumulator.
/// Formula: `alpha(t) = min_alpha + (1 - min_alpha) * 0.5 * (1 + cos(2π * frequency * t))`
#[derive(Component, Debug, Clone, Copy)]
pub struct PhantomFlicker {
    /// Flicker frequency in Hz. Full alpha period = `1.0 / frequency`.
    pub frequency: f32,
    /// Minimum alpha during flicker. Valid range `[0.0, 1.0]`.
    pub min_alpha: f32,
}

impl Default for PhantomFlicker {
    fn default() -> Self {
        Self {
            frequency: 4.0,
            min_alpha: 0.3,
        }
    }
}
