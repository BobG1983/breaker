//! Chips domain plugin — Amps, Augments, and Overclocks system.

mod definition;
mod plugin;
mod resources;

pub(crate) use definition::{ChipDefinition, ChipKind};
pub(crate) use plugin::ChipsPlugin;
pub(crate) use resources::ChipRegistry;
