//! Chips domain plugin — Amps, Augments, and Overclocks system.

pub(crate) mod components;
pub(crate) mod definition;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use definition::{ChipDefinition, ChipKind};
pub(crate) use plugin::ChipsPlugin;
pub(crate) use resources::ChipRegistry;
