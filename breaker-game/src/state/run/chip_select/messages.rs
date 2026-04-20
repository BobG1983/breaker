//! Messages sent by the UI domain.

use bevy::prelude::*;

/// Sent when the player selects a chip from the selection screen.
///
/// Consumed by the chips plugin (applies effects).
#[derive(Message, Clone, Debug)]
pub struct ChipSelected {
    /// Display name of the selected chip.
    pub name: String,
}

impl std::fmt::Display for ChipSelected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Sent when the player skips a chip offering on the selection screen.
///
/// Consumed by the protocol domain's `greed_on_skip` system (increments
/// `GreedStacks.skips`). The message itself is unit-payload — the receiver
/// only needs to know a skip occurred this tick.
#[derive(Message, Clone, Debug)]
pub struct ChipOfferSkipped;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_selected_debug_format() {
        let msg = ChipSelected {
            name: "Piercing Shot".to_owned(),
        };
        assert!(format!("{msg:?}").contains("ChipSelected"));
    }

    #[test]
    fn chip_selected_display_format() {
        let msg = ChipSelected {
            name: "Piercing Shot".to_owned(),
        };
        assert_eq!(format!("{msg}"), "Piercing Shot");
    }
}
