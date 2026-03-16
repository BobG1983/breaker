//! Upgrade selection screen components.

use bevy::prelude::*;

/// Marker component on the root upgrade-select UI entity.
#[derive(Component)]
pub struct UpgradeSelectScreen;

/// Identifies an upgrade card by its index (0, 1, 2).
#[derive(Component, Debug, Clone, Copy)]
pub struct UpgradeCard {
    /// Zero-based index of this card.
    pub index: usize,
}

/// Marker on the timer text entity so `update_upgrade_display` can find it.
#[derive(Component)]
pub struct UpgradeTimerText;
