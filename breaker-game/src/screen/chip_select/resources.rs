//! Chip selection screen resources.

use bevy::prelude::*;
use breaker_derive::GameConfig;
use serde::Deserialize;

use crate::chips::ChipDefinition;

/// Chip select defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "ChipSelectConfig")]
pub(crate) struct ChipSelectDefaults {
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
    /// Number of chips to offer per node.
    #[serde(default = "default_offers_per_node")]
    pub offers_per_node: usize,
    /// Base weight for Common rarity chips.
    #[serde(default = "default_rarity_weight_common")]
    pub rarity_weight_common: f32,
    /// Base weight for Uncommon rarity chips.
    #[serde(default = "default_rarity_weight_uncommon")]
    pub rarity_weight_uncommon: f32,
    /// Base weight for Rare rarity chips.
    #[serde(default = "default_rarity_weight_rare")]
    pub rarity_weight_rare: f32,
    /// Base weight for Legendary rarity chips.
    #[serde(default = "default_rarity_weight_legendary")]
    pub rarity_weight_legendary: f32,
    /// Weight decay factor for chips offered but not selected.
    #[serde(default = "default_seen_decay_factor")]
    pub seen_decay_factor: f32,
    /// RGB color for Common rarity card border.
    #[serde(default = "default_rarity_color_common")]
    pub rarity_color_common_rgb: [f32; 3],
    /// RGB color for Uncommon rarity card border.
    #[serde(default = "default_rarity_color_uncommon")]
    pub rarity_color_uncommon_rgb: [f32; 3],
    /// RGB color for Rare rarity card border.
    #[serde(default = "default_rarity_color_rare")]
    pub rarity_color_rare_rgb: [f32; 3],
    /// RGB color for Legendary rarity card border.
    #[serde(default = "default_rarity_color_legendary")]
    pub rarity_color_legendary_rgb: [f32; 3],
}

fn default_offers_per_node() -> usize {
    3
}
fn default_rarity_weight_common() -> f32 {
    100.0
}
fn default_rarity_weight_uncommon() -> f32 {
    50.0
}
fn default_rarity_weight_rare() -> f32 {
    15.0
}
fn default_rarity_weight_legendary() -> f32 {
    3.0
}
fn default_seen_decay_factor() -> f32 {
    0.8
}
fn default_rarity_color_common() -> [f32; 3] {
    [0.5, 0.5, 0.5]
}
fn default_rarity_color_uncommon() -> [f32; 3] {
    [0.2, 0.8, 0.3]
}
fn default_rarity_color_rare() -> [f32; 3] {
    [0.3, 0.4, 1.0]
}
fn default_rarity_color_legendary() -> [f32; 3] {
    [1.0, 0.85, 0.2]
}

impl Default for ChipSelectDefaults {
    fn default() -> Self {
        Self {
            timer_secs: 10.0,
            card_title_font_size: 36.0,
            card_description_font_size: 20.0,
            timer_font_size: 48.0,
            selected_color_rgb: [0.4, 0.8, 1.0],
            normal_color_rgb: [0.3, 0.3, 0.4],
            timer_color_rgb: [1.0, 0.8, 0.2],
            offers_per_node: default_offers_per_node(),
            rarity_weight_common: default_rarity_weight_common(),
            rarity_weight_uncommon: default_rarity_weight_uncommon(),
            rarity_weight_rare: default_rarity_weight_rare(),
            rarity_weight_legendary: default_rarity_weight_legendary(),
            seen_decay_factor: default_seen_decay_factor(),
            rarity_color_common_rgb: default_rarity_color_common(),
            rarity_color_uncommon_rgb: default_rarity_color_uncommon(),
            rarity_color_rare_rgb: default_rarity_color_rare(),
            rarity_color_legendary_rgb: default_rarity_color_legendary(),
        }
    }
}

/// Screen-local countdown timer for the chip selection screen.
#[derive(Resource, Debug)]
pub(super) struct ChipSelectTimer {
    /// Remaining time in seconds.
    pub remaining: f32,
}

/// Tracks which card is currently highlighted.
#[derive(Resource, Debug)]
pub(super) struct ChipSelectSelection {
    /// Zero-based index of the selected card.
    pub index: usize,
}

/// The chip definitions offered this screen visit.
///
/// Inserted by `generate_chip_offerings`, read by `spawn_chip_select`
/// and `handle_chip_input` to resolve a selection index into chip identity.
#[derive(Resource, Debug)]
pub struct ChipOffers(pub Vec<ChipDefinition>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_positive_timer() {
        let config = ChipSelectConfig::default();
        assert!(config.timer_secs > 0.0);
    }

    #[test]
    fn chip_select_defaults_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/config/defaults.chipselect.ron"
        ));
        let result: ChipSelectDefaults =
            ron::de::from_str(ron_str).expect("chipselect RON should parse");
        assert!(result.timer_secs > 0.0);
    }
}
