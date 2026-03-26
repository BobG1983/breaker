//! Cells domain plugin — cell types, damage handling, destruction.

pub(crate) mod components;
pub(crate) mod definition;
pub(crate) mod filters;
pub(crate) mod messages;
mod plugin;
pub(crate) mod queries;
pub(crate) mod resources;
mod systems;

pub(crate) use definition::CellTypeDefinition;
pub(crate) use plugin::CellsPlugin;
pub(crate) use resources::{CellConfig, CellDefaults, CellTypeRegistry};
pub(crate) use systems::handle_cell_hit;
