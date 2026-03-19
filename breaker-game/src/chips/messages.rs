//! Chip domain events — internal triggers for observer-based effect handlers.

use bevy::prelude::*;

use super::definition::ChipEffect;

/// Triggered when a chip effect should be applied.
///
/// Dispatched by `apply_chip_effect` for each selected chip.
/// Each per-effect observer self-selects via pattern matching on `effect`.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChipEffectApplied {
    /// The effect to apply.
    pub effect: ChipEffect,
    /// Maximum stacks for this chip.
    pub max_stacks: u32,
}
