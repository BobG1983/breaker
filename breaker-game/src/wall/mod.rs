//! Wall domain plugin — invisible boundary entities for CCD collision.

pub(crate) mod builder;
pub mod components;
pub mod definition;
pub(crate) mod messages;
mod plugin;
/// Wall registry — `SeedableRegistry` for `WallDefinition` RON assets.
pub mod registry;
pub(crate) mod systems;

pub use definition::WallDefinition;
pub(crate) use plugin::WallPlugin;
pub use registry::WallRegistry;
