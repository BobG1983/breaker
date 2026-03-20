//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub(crate) mod behaviors;
pub mod components;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod resources;
pub mod sets;
pub mod systems;

pub use plugin::BoltPlugin;
pub use resources::{BoltConfig, BoltDefaults};
pub use sets::BoltSystems;
