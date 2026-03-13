//! Cells domain plugin — cell types, damage handling, destruction.

pub mod components;
pub mod messages;
mod plugin;
pub mod queries;
pub mod resources;
mod systems;

pub use plugin::CellsPlugin;
pub use resources::{CellConfig, CellDefaults, CellTypeDefinition, CellTypeRegistry};
