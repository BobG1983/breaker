//! `EffectSourceChip` — tracks which chip sourced an effect entity.

use bevy::prelude::*;

/// Identifies which chip (upgrade) caused this effect entity to be spawned.
///
/// Used for damage attribution and UI display.
/// `None` indicates the effect was not chip-sourced (e.g., from a cell death cascade).
#[derive(Component)]
pub struct EffectSourceChip(pub Option<String>);
