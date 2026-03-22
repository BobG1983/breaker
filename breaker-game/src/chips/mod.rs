//! Chips domain plugin — Amps, Augments, and Overclocks system.

pub(crate) mod components;
pub mod definition;
pub(crate) mod effects;
pub mod inventory;
pub(crate) mod offering;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use definition::ChipDefinition;
pub use definition::{ImpactTarget, TriggerChain};
pub(crate) use plugin::ChipsPlugin;
pub(crate) use resources::ChipRegistry;
