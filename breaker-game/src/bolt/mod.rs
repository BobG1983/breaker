//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub(crate) mod builder;
pub mod components;
pub mod definition;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod registry;
pub mod resources;
pub mod sets;
pub mod systems;

pub use definition::BoltDefinition;
pub use plugin::BoltPlugin;
pub use registry::BoltRegistry;
pub use resources::{
    BASE_BOLT_DAMAGE, BoltConfig, BoltDefaults, DEFAULT_BOLT_ANGLE_SPREAD,
    DEFAULT_BOLT_BASE_DAMAGE, DEFAULT_BOLT_SPAWN_OFFSET_Y,
};
pub use sets::BoltSystems;
