//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub(crate) mod builder;
pub mod components;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod resources;
pub mod sets;
pub mod systems;

pub use plugin::BoltPlugin;
pub use resources::{BASE_BOLT_DAMAGE, BoltConfig, BoltDefaults};
pub use sets::BoltSystems;
