//! Chip selection sub-domain — placeholder between-node chip screen.

mod components;
pub mod messages;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use components::ChipSelectScreen;
pub(crate) use plugin::ChipSelectPlugin;
pub use resources::{ChipOffering, ChipOffers};
pub(crate) use resources::{ChipSelectConfig, ChipSelectDefaults};

/// Convert an `[f32; 3]` RGB config array to a Bevy `Color`.
pub(crate) const fn color_from_rgb(rgb: [f32; 3]) -> bevy::prelude::Color {
    bevy::prelude::Color::srgb(rgb[0], rgb[1], rgb[2])
}
