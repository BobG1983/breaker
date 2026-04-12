//! Chip selection screen resources.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;

use crate::chips::{ChipDefinition, definition::EvolutionIngredient};

/// Chip select configuration resource.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "ChipSelectDefaults",
    path = "config/defaults.chipselect.ron",
    ext = "chipselect.ron"
)]
pub(crate) struct ChipSelectConfig {
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
    pub offers_per_node: usize,
    /// Base weight for Common rarity chips.
    pub rarity_weight_common: f32,
    /// Base weight for Uncommon rarity chips.
    pub rarity_weight_uncommon: f32,
    /// Base weight for Rare rarity chips.
    pub rarity_weight_rare: f32,
    /// Base weight for Legendary rarity chips.
    pub rarity_weight_legendary: f32,
    /// Weight decay factor for chips offered but not selected.
    pub seen_decay_factor: f32,
    /// RGB color for Common rarity card border.
    pub rarity_color_common_rgb: [f32; 3],
    /// RGB color for Uncommon rarity card border.
    pub rarity_color_uncommon_rgb: [f32; 3],
    /// RGB color for Rare rarity card border.
    pub rarity_color_rare_rgb: [f32; 3],
    /// RGB color for Legendary rarity card border.
    pub rarity_color_legendary_rgb: [f32; 3],
}

impl Default for ChipSelectConfig {
    fn default() -> Self {
        Self {
            timer_secs: 10.0,
            card_title_font_size: 36.0,
            card_description_font_size: 20.0,
            timer_font_size: 48.0,
            selected_color_rgb: [0.4, 0.8, 1.0],
            normal_color_rgb: [0.3, 0.3, 0.4],
            timer_color_rgb: [1.0, 0.8, 0.2],
            offers_per_node: 3,
            rarity_weight_common: 100.0,
            rarity_weight_uncommon: 50.0,
            rarity_weight_rare: 15.0,
            rarity_weight_legendary: 3.0,
            seen_decay_factor: 0.8,
            rarity_color_common_rgb: [0.5, 0.5, 0.5],
            rarity_color_uncommon_rgb: [0.2, 0.8, 0.3],
            rarity_color_rare_rgb: [0.3, 0.4, 1.0],
            rarity_color_legendary_rgb: [1.0, 0.85, 0.2],
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

/// A single offering on the chip selection screen — either a normal chip
/// or an evolution that combines existing chips into a new one.
#[derive(Debug, Clone)]
pub enum ChipOffering {
    /// A standard chip offered for selection.
    Normal(ChipDefinition),
    /// An evolution that consumes ingredient stacks to produce a new chip.
    Evolution {
        /// Ingredients consumed from the player's inventory.
        ingredients: Vec<EvolutionIngredient>,
        /// The chip produced by this evolution.
        result: ChipDefinition,
    },
}

impl ChipOffering {
    /// Returns the display name of this offering.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Normal(def) => &def.name,
            Self::Evolution { result, .. } => &result.name,
        }
    }

    /// Returns the chip definition this offering would grant.
    #[must_use]
    pub const fn definition(&self) -> &ChipDefinition {
        match self {
            Self::Normal(def) | Self::Evolution { result: def, .. } => def,
        }
    }
}

/// The chip offerings presented this screen visit.
///
/// Inserted by `generate_chip_offerings`, read by `spawn_chip_select`
/// and `handle_chip_input` to resolve a selection index into chip identity.
#[derive(Resource, Debug)]
pub struct ChipOffers(pub Vec<ChipOffering>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chips::definition::EvolutionIngredient,
        effect_v3::{
            effects::PiercingConfig,
            types::{EffectType, Tree},
        },
    };

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

    // --- ChipOffering::Normal ---

    #[test]
    fn normal_offering_name_returns_inner_definition_name() {
        let def = ChipDefinition::test(
            "Piercing Shot",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        );
        let offering = ChipOffering::Normal(def);
        assert_eq!(offering.name(), "Piercing Shot");
    }

    #[test]
    fn normal_offering_definition_returns_inner_definition() {
        let def = ChipDefinition::test(
            "Piercing Shot",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        );
        let offering = ChipOffering::Normal(def);
        assert_eq!(offering.definition().name, "Piercing Shot");
    }

    // --- ChipOffering::Evolution ---

    #[test]
    fn evolution_offering_name_returns_result_name() {
        let offering = ChipOffering::Evolution {
            ingredients: vec![EvolutionIngredient {
                chip_name: "A".to_owned(),
                stacks_required: 2,
            }],
            result: ChipDefinition::test(
                "A+",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
                1,
            ),
        };
        assert_eq!(offering.name(), "A+");
    }

    #[test]
    fn evolution_offering_definition_returns_result_definition() {
        let offering = ChipOffering::Evolution {
            ingredients: vec![EvolutionIngredient {
                chip_name: "A".to_owned(),
                stacks_required: 2,
            }],
            result: ChipDefinition::test(
                "Barrage",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
                1,
            ),
        };
        assert_eq!(offering.definition().name, "Barrage");
    }
}
