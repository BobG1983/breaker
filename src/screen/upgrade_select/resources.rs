//! Upgrade selection screen resources.

use bevy::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

use crate::upgrades::UpgradeDefinition;

/// Upgrade select defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "UpgradeSelectConfig")]
pub struct UpgradeSelectDefaults {
    /// Time in seconds for the selection countdown.
    pub timer_secs: f32,
    /// Font size for card title text.
    pub card_title_font_size: f32,
    /// Font size for card description text.
    pub card_description_font_size: f32,
    /// Font size for the countdown timer display.
    pub timer_font_size: f32,
    /// RGB color for the selected card border.
    pub selected_color_rgb: [f32; 3],
    /// RGB color for unselected card borders.
    pub normal_color_rgb: [f32; 3],
    /// RGB color for the timer text.
    pub timer_color_rgb: [f32; 3],
}

impl Default for UpgradeSelectDefaults {
    fn default() -> Self {
        Self {
            timer_secs: 10.0,
            card_title_font_size: 36.0,
            card_description_font_size: 20.0,
            timer_font_size: 48.0,
            selected_color_rgb: [0.4, 0.8, 1.0],
            normal_color_rgb: [0.3, 0.3, 0.4],
            timer_color_rgb: [1.0, 0.8, 0.2],
        }
    }
}

/// Screen-local countdown timer for the upgrade selection screen.
#[derive(Resource, Debug)]
pub struct UpgradeSelectTimer {
    /// Remaining time in seconds.
    pub remaining: f32,
}

/// Tracks which card is currently highlighted.
#[derive(Resource, Debug)]
pub struct UpgradeSelectSelection {
    /// Zero-based index of the selected card.
    pub index: usize,
}

/// The upgrade definitions offered this screen visit.
///
/// Inserted by `spawn_upgrade_select`, read by `handle_upgrade_input`
/// to resolve a selection index into upgrade identity.
#[derive(Resource, Debug)]
pub struct UpgradeOffers(pub Vec<UpgradeDefinition>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_positive_timer() {
        let config = UpgradeSelectConfig::default();
        assert!(config.timer_secs > 0.0);
    }

    #[test]
    fn upgrade_select_defaults_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/config/defaults.upgradeselect.ron"
        ));
        let result: UpgradeSelectDefaults =
            ron::de::from_str(ron_str).expect("upgradeselect RON should parse");
        assert!(result.timer_secs > 0.0);
    }
}
