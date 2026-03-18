//! Wall domain plugin — invisible boundary entities for CCD collision.

pub(crate) mod components;
mod plugin;
pub(crate) mod systems;

pub(crate) use plugin::WallPlugin;
