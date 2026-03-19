//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::Deserialize;

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CellTypeDefinition {
    /// Unique identifier.
    pub id: String,
    /// Single-char alias used in node layout grids.
    pub alias: char,
    /// Hit points for this cell type.
    pub hp: f32,
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
