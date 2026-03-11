//! Cells domain resources.

use bevy::prelude::*;
use serde::Deserialize;

/// Configuration for cell mechanics.
#[derive(Resource, Debug, Clone)]
pub struct CellConfig {
    /// Half-width of a cell in world units.
    pub half_width: f32,
    /// Half-height of a cell in world units.
    pub half_height: f32,
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

impl Default for CellConfig {
    fn default() -> Self {
        CellDefaults::default().into()
    }
}

/// Cell defaults loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CellDefaults {
    /// Half-width of a cell in world units.
    pub half_width: f32,
    /// Half-height of a cell in world units.
    pub half_height: f32,
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
            half_width: 35.0,
            half_height: 12.0,
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

impl From<CellDefaults> for CellConfig {
    fn from(d: CellDefaults) -> Self {
        Self {
            half_width: d.half_width,
            half_height: d.half_height,
            padding_x: d.padding_x,
            padding_y: d.padding_y,
            grid_cols: d.grid_cols,
            grid_rows: d.grid_rows,
            grid_top_offset: d.grid_top_offset,
            standard_hp: d.standard_hp,
            tough_hp: d.tough_hp,
            standard_color_rgb: d.standard_color_rgb,
            tough_color_rgb: d.tough_color_rgb,
            tough_row_index: d.tough_row_index,
            damage_hdr_base: d.damage_hdr_base,
            damage_green_min: d.damage_green_min,
            damage_blue_range: d.damage_blue_range,
            damage_blue_base: d.damage_blue_base,
        }
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
