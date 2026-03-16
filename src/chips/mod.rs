//! Chips domain plugin — Amps, Augments, and Overclocks system.

mod definition;
mod plugin;
mod resources;

pub use definition::{ChipDefinition, ChipKind};
pub use plugin::ChipsPlugin;
pub use resources::ChipRegistry;
