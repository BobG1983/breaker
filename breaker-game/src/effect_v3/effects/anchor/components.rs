//! Anchor runtime components.

use bevy::prelude::*;

/// Marks the breaker as having an active anchor effect and holds its parameters.
#[derive(Component, Debug, Clone)]
pub struct AnchorActive {
    /// Multiplier applied to bump force while anchored.
    pub bump_force_multiplier:     f32,
    /// Multiplier applied during the perfect-hit window.
    pub perfect_window_multiplier: f32,
    /// Delay in seconds before the anchor plants after activation.
    pub plant_delay:               f32,
}

/// Countdown timer tracking how long until the anchor plants.
#[derive(Component, Debug, Clone)]
pub struct AnchorTimer(pub f32);

/// Marker indicating the anchor has planted (breaker is now stationary).
#[derive(Component, Debug, Clone)]
pub struct AnchorPlanted;
