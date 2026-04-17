//! Cross-domain ordering anchors for the chip-select screen.
//!
//! Other domains (e.g. `protocol`) order their own systems against these sets
//! instead of referencing bare chip-select function symbols, keeping the
//! coupling surface stable if the underlying systems are renamed or split.

use bevy::prelude::*;

/// Ordering anchors exposed by the chip-select domain.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ChipSelectSystems {
    /// Tags `generate_chip_offerings`.
    GenerateOfferings,
    /// Tags `spawn_chip_select`.
    SpawnScreen,
    /// Tags `handle_chip_input`.
    HandleInput,
}
