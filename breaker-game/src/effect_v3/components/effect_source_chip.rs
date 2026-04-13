//! `EffectSourceChip` — tracks which chip sourced an effect entity.

use bevy::prelude::*;

/// Identifies which chip (upgrade) caused this effect entity to be spawned.
///
/// Used for damage attribution and UI display.
/// `None` indicates the effect was not chip-sourced (e.g., from a cell death cascade).
#[derive(Component)]
pub struct EffectSourceChip(pub Option<String>);

impl EffectSourceChip {
    /// Constructs an `EffectSourceChip` from a `fire()` source string.
    /// Empty source strings map to `None` (non-chip-sourced).
    #[must_use]
    pub fn from_source(source: &str) -> Self {
        Self((!source.is_empty()).then(|| source.to_owned()))
    }
}
