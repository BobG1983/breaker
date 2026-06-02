//! Chip selection sub-domain — placeholder between-node chip screen.

mod components;
pub mod messages;
mod plugin;
mod resources;
pub(crate) mod sets;
pub(crate) mod systems;

#[cfg(test)]
mod schedule_tests;

pub(crate) use components::ChipSelectScreen;
pub(crate) use plugin::ChipSelectPlugin;
pub use resources::{ChipOffering, ChipOffers};
pub(crate) use resources::{ChipSelectConfig, ChipSelectDefaults};
