//! Cells domain plugin — cell types, damage handling, destruction.

pub(crate) mod components;
pub(crate) mod messages;
mod plugin;
pub(crate) mod queries;
pub(crate) mod resources;
mod systems;

pub(crate) use plugin::CellsPlugin;
pub(crate) use resources::{CellConfig, CellDefaults, CellTypeDefinition, CellTypeRegistry};
