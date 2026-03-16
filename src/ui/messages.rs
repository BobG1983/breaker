//! Messages sent by the UI domain.

use bevy::prelude::*;

use crate::upgrades::UpgradeKind;

/// Sent when the player selects an upgrade from the selection screen.
///
/// Consumed by the upgrades plugin (applies effects).
#[derive(Message, Clone, Debug)]
#[allow(dead_code)]
pub struct UpgradeSelected {
    /// Display name of the selected upgrade.
    pub name: String,
    /// Category of the selected upgrade.
    pub kind: UpgradeKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_selected_debug_format() {
        let msg = UpgradeSelected {
            name: "Piercing Shot".to_owned(),
            kind: UpgradeKind::Amp,
        };
        assert!(format!("{msg:?}").contains("UpgradeSelected"));
    }
}
