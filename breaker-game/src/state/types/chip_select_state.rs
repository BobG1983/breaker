//! Chip select state — sub-state of [`RunPhase::ChipSelect`].

use bevy::prelude::*;

use super::RunPhase;

/// Chip selection lifecycle state.
///
/// Sub-state of [`RunPhase::ChipSelect`]. Controls the chip selection
/// screen between nodes.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunPhase = RunPhase::ChipSelect)]
pub enum ChipSelectState {
    /// Chip select loading (pass-through).
    #[default]
    Loading,
    /// Animate chip select entrance (pass-through until transitions are wired).
    AnimateIn,
    /// Player is selecting a chip.
    Selecting,
    /// Animate chip select exit (pass-through until transitions are wired).
    AnimateOut,
    /// Chip select teardown — parent `RunPhase` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_chip_select_state_is_loading() {
        assert_eq!(ChipSelectState::default(), ChipSelectState::Loading);
    }
}
