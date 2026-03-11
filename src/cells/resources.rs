//! Cells domain resources.

use bevy::prelude::*;

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
    /// Green channel range added based on health fraction.
    pub damage_green_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
}

impl CellConfig {
    /// Standard cell color as a Bevy [`Color`].
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn standard_color(&self) -> Color {
        Color::srgb(
            self.standard_color_rgb[0],
            self.standard_color_rgb[1],
            self.standard_color_rgb[2],
        )
    }

    /// Tough cell color as a Bevy [`Color`].
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn tough_color(&self) -> Color {
        Color::srgb(
            self.tough_color_rgb[0],
            self.tough_color_rgb[1],
            self.tough_color_rgb[2],
        )
    }
}

impl Default for CellConfig {
    fn default() -> Self {
        crate::screen::defaults::CellDefaults::default().into()
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
}
