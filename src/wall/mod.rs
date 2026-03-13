//! Wall domain plugin — invisible boundary entities for CCD collision.

pub mod components;
mod plugin;
pub mod systems;

pub use plugin::WallPlugin;
