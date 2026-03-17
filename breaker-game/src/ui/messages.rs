//! Messages sent by the UI domain.

use bevy::prelude::*;

use crate::chips::ChipKind;

/// Sent when the player selects a chip from the selection screen.
///
/// Consumed by the chips plugin (applies effects).
#[derive(Message, Clone, Debug)]
#[allow(dead_code)]
pub struct ChipSelected {
    /// Display name of the selected chip.
    pub name: String,
    /// Category of the selected chip.
    pub kind: ChipKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_selected_debug_format() {
        let msg = ChipSelected {
            name: "Piercing Shot".to_owned(),
            kind: ChipKind::Amp,
        };
        assert!(format!("{msg:?}").contains("ChipSelected"));
    }
}
