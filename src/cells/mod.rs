//! Cells domain plugin — cell types, grid layout, destruction.

pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
mod systems;

pub use plugin::CellsPlugin;
pub use resources::{CellConfig, CellDefaults};
