//! Messages sent by the UI domain.

use bevy::prelude::*;

/// Sent when the player selects an upgrade from the selection screen.
///
/// Consumed by the upgrades plugin (applies effects).
#[derive(Message, Clone, Debug)]
pub struct UpgradeSelected {
    /// Index of the selected upgrade option (0-based).
    pub choice: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_selected_debug_format() {
        let msg = UpgradeSelected { choice: 0 };
        assert!(format!("{msg:?}").contains("UpgradeSelected"));
    }
}
