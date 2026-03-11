//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub mod components;
pub mod filters;
mod plugin;
pub mod resources;
pub mod systems;

pub use plugin::BoltPlugin;
pub use resources::{BoltConfig, BoltDefaults};
