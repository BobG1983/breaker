//! Cells domain resources.

use bevy::prelude::*;
use brickbreaker_derive::GameConfig;
use serde::Deserialize;

/// Cell defaults loaded from RON.
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
    /// Number of columns in the grid.
    pub grid_cols: u32,
    /// Number of rows in the grid.
    pub grid_rows: u32,
    /// Y offset from playfield top for grid start.
    pub grid_top_offset: f32,
    /// HP for standard cells.
    pub standard_hp: u32,
    /// HP for tough cells.
    pub tough_hp: u32,
    /// RGB values for standard cell HDR color.
    pub standard_color_rgb: [f32; 3],
    /// RGB values for tough cell HDR color.
    pub tough_color_rgb: [f32; 3],
    /// Row index (0-indexed from top) that contains tough cells.
    pub tough_row_index: u32,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base: f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min: f32,
    /// Blue channel range added based on health fraction.
    pub damage_blue_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
}

impl Default for CellDefaults {
    fn default() -> Self {
        Self {
            width: 70.0,
            height: 24.0,
            padding_x: 4.0,
            padding_y: 4.0,
            grid_cols: 10,
            grid_rows: 5,
            grid_top_offset: 50.0,
            standard_hp: 1,
            tough_hp: 3,
            standard_color_rgb: [4.0, 0.2, 0.5],
            tough_color_rgb: [2.5, 0.2, 4.0],
            tough_row_index: 0,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        }
    }
}

impl CellConfig {
    /// Standard cell color as a Bevy [`Color`].
    #[must_use]
    pub fn standard_color(&self) -> Color {
        crate::shared::color_from_rgb(self.standard_color_rgb)
    }

    /// Tough cell color as a Bevy [`Color`].
    #[must_use]
    pub fn tough_color(&self) -> Color {
        crate::shared::color_from_rgb(self.tough_color_rgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid_dimensions_positive() {
        let config = CellConfig::default();
        assert!(config.grid_cols > 0);
        assert!(config.grid_rows > 0);
    }

    #[test]
    fn tough_hp_exceeds_standard() {
        let config = CellConfig::default();
        assert!(config.tough_hp > config.standard_hp);
    }

    #[test]
    fn cell_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.cells.ron");
        let result: CellDefaults = ron::de::from_str(ron_str).expect("cells RON should parse");
        assert!(result.grid_cols > 0);
    }
}
