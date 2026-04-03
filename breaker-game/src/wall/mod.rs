//! Wall domain plugin — invisible boundary entities for CCD collision.

pub mod components;
pub mod definition;
pub(crate) mod filters;
pub(crate) mod messages;
mod plugin;
pub mod registry;
pub(crate) mod systems;

pub use definition::WallDefinition;
pub(crate) use plugin::WallPlugin;
pub use registry::WallRegistry;
