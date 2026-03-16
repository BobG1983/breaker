//! Chip selection screen components.

use bevy::prelude::*;

/// Marker component on the root chip-select UI entity.
#[derive(Component)]
pub struct ChipSelectScreen;

/// Identifies an chip card by its index (0, 1, 2).
#[derive(Component, Debug, Clone, Copy)]
pub struct ChipCard {
    /// Zero-based index of this card.
    pub index: usize,
}

/// Marker on the timer text entity so `update_chip_display` can find it.
#[derive(Component)]
pub struct ChipTimerText;
