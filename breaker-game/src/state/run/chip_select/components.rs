//! Chip selection screen components.

use bevy::prelude::*;

/// Marker component on the root chip-select UI entity.
#[derive(Component)]
pub(crate) struct ChipSelectScreen;

/// Identifies a chip card by its index (0, 1, 2).
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ChipCard {
    /// Zero-based index of this card.
    pub(super) index: usize,
}

/// Marker on the timer text entity so `update_chip_display` can find it.
#[derive(Component)]
pub(super) struct ChipTimerText;

/// Marker on the protocol-offer UI entity so tests can count it and layout
/// code can target it. `None` offer means no entity is spawned.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ProtocolCard;

/// Marker on the Greed Skip-row root entity. Spawned by `spawn_chip_select`
/// only when `ActiveProtocols::contains(ProtocolKind::Greed)` is true.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SkipButton;

/// Marker on the Skip-row indicator-text entity. Its `Text` reports the
/// player's current skip count and the projected rarity boost for the next
/// chip offering. Computed by `format_skip_indicator` at spawn time.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct SkipIndicator;
