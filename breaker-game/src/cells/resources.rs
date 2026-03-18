//! Cells domain resources.

use std::collections::HashMap;

use bevy::prelude::*;
use breaker_derive::GameConfig;
use serde::Deserialize;

/// Cell defaults loaded from RON — shared grid layout properties only.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "CellConfig")]
pub struct CellDefaults {
    /// Full width of a cell in world units.
    pub width: f32,
    /// Full height of a cell in world units.
    pub height: f32,
    /// Horizontal padding between cells.
    pub padding_x: f32,
    /// Vertical padding between cells.
    pub padding_y: f32,
}

impl Default for CellDefaults {
    fn default() -> Self {
        Self {
            width: 70.0,
            height: 24.0,
            padding_x: 4.0,
            padding_y: 4.0,
        }
    }
}

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CellTypeDefinition {
    /// Unique identifier.
    pub id: String,
    /// Single-char alias used in node layout grids.
    pub alias: char,
    /// Hit points for this cell type.
    pub hp: u32,
    /// HDR RGB color.
    pub color_rgb: [f32; 3],
    /// Whether this cell counts toward node completion.
    pub required_to_clear: bool,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base: f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min: f32,
    /// Blue channel range added based on health fraction.
    pub damage_blue_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
}

impl CellTypeDefinition {
    /// Cell color as a Bevy [`Color`].
    #[must_use]
    pub const fn color(&self) -> Color {
        crate::shared::color_from_rgb(self.color_rgb)
    }
}

/// Registry mapping alias chars to cell type definitions. Built at boot from all loaded RONs.
#[derive(Resource, Debug, Default)]
pub struct CellTypeRegistry {
    /// Map from alias char to cell type definition.
    pub types: HashMap<char, CellTypeDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_defaults_width_height_positive() {
        let config = CellConfig::default();
        assert!(config.width > 0.0);
        assert!(config.height > 0.0);
    }

    #[test]
    fn cell_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.cells.ron");
        let result: CellDefaults = ron::de::from_str(ron_str).expect("cells RON should parse");
        assert!(result.width > 0.0);
    }

    #[test]
    fn all_cell_type_rons_parse() {
        use std::fs;
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cells");
        for entry in fs::read_dir(dir).expect("assets/cells/ should exist") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("ron") {
                let content = fs::read_to_string(&path).unwrap();
                let def: CellTypeDefinition = ron::de::from_str(&content).unwrap_or_else(|e| {
                    panic!("{}: {e}", path.display());
                });
                assert!(
                    !def.id.is_empty(),
                    "{}: id must not be empty",
                    path.display()
                );
                assert!(
                    def.alias != '.',
                    "{}: alias must not be '.'",
                    path.display()
                );
                assert!(def.hp > 0, "{}: hp must be > 0", path.display());
            }
        }
    }

    #[test]
    fn registry_detects_duplicate_aliases() {
        let def_a = CellTypeDefinition {
            id: "a".to_owned(),
            alias: 'S',
            hp: 1,
            color_rgb: [1.0, 0.0, 0.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        };
        let def_b = CellTypeDefinition {
            id: "b".to_owned(),
            alias: 'S',
            hp: 2,
            color_rgb: [0.0, 1.0, 0.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        };
        let mut registry = CellTypeRegistry::default();
        registry.types.insert(def_a.alias, def_a);
        let had_existing = registry.types.insert(def_b.alias, def_b);
        assert!(
            had_existing.is_some(),
            "inserting duplicate alias should replace"
        );
    }
}
