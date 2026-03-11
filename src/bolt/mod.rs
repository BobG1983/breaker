//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub mod components;
mod plugin;
mod resources;
pub mod systems;

pub use plugin::BoltPlugin;
pub use resources::BoltConfig;
