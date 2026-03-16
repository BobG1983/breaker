//! Run setup screen components.

use bevy::prelude::*;

/// Marker component on the root run-setup UI entity.
#[derive(Component)]
pub struct RunSetupScreen;

/// Identifies a breaker card in the selection screen.
#[derive(Component, Debug)]
pub struct BreakerCard {
    /// The archetype name this card represents.
    pub archetype_name: String,
}
