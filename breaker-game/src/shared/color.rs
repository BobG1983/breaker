//! Color conversion utilities.

use bevy::prelude::Color;

/// Converts an `[f32; 3]` RGB triple into an sRGB [`Color`].
#[must_use]
pub const fn color_from_rgb(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}
