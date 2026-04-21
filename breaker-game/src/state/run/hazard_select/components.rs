//! Hazard selection screen components.

use bevy::prelude::*;

/// Marker component on the root hazard-select UI entity.
#[derive(Component)]
pub(crate) struct HazardSelectScreen;

/// Identifies a hazard card by its index (0, 1, 2). `update_hazard_display`
/// reads `index` each frame to compare against `HazardSelectSelection`
/// (parallel to `ChipCard`).
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct HazardCard {
    /// Zero-based index of this card.
    pub(crate) index: usize,
}

/// Marker on the timer text entity so tests can find it.
#[derive(Component)]
pub(crate) struct HazardTimerText;
