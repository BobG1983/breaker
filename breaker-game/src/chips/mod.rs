//! Chips domain plugin — passive and triggered chip effects.

pub(crate) mod components;
pub mod definition;
pub mod inventory;
pub(crate) mod offering;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use definition::ChipDefinition;
pub(crate) use plugin::ChipsPlugin;
pub use resources::ChipCatalog;
#[cfg(test)]
pub(crate) use resources::Recipe;
pub(crate) use resources::{ChipTemplateRegistry, EvolutionTemplateRegistry};
