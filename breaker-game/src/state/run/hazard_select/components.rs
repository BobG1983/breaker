//! Hazard selection screen components.

use bevy::prelude::*;

/// Marker component on the root hazard-select UI entity.
#[derive(Component)]
pub(crate) struct HazardSelectScreen;

/// Identifies a hazard card by its index (0, 1, 2).
///
/// The `index` field is only consulted by tests that verify the spawn
/// pattern (cards {0, 1, 2}; first card starts selected). Production
/// selection logic consults `HazardSelectSelection` and the UI row order
/// directly, so the field is gated off in non-test builds.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct HazardCard {
    /// Zero-based index of this card.
    #[cfg(test)]
    pub(crate) index: usize,
}

/// Marker on the timer text entity so tests can find it.
#[derive(Component)]
pub(crate) struct HazardTimerText;
