//! Messages sent by the UI domain.

use bevy::prelude::*;

use crate::chips::ChipKind;

/// Sent when the player selects a chip from the selection screen.
///
/// Consumed by the chips plugin (applies effects).
#[derive(Message, Clone, Debug)]
pub(crate) struct ChipSelected {
    /// Display name of the selected chip.
    pub name: String,
    /// Category of the selected chip.
    pub kind: ChipKind,
}

impl std::fmt::Display for ChipSelected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.name, self.kind)
    }
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

    #[test]
    fn chip_selected_display_format() {
        let msg = ChipSelected {
            name: "Piercing Shot".to_owned(),
            kind: ChipKind::Amp,
        };
        assert_eq!(format!("{msg}"), "Piercing Shot (Amp)");
    }
}
