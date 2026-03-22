//! Chip selection sub-domain — placeholder between-node chip screen.

mod components;
mod plugin;
mod resources;
mod systems;

pub(crate) use components::ChipSelectScreen;
pub(crate) use plugin::ChipSelectPlugin;
pub use resources::ChipOffers;
pub(crate) use resources::{ChipSelectConfig, ChipSelectDefaults};
