//! Wall domain plugin — invisible boundary entities for CCD collision.

pub(crate) mod components;
pub(crate) mod filters;
pub(crate) mod messages;
mod plugin;
pub(crate) mod systems;

pub(crate) use plugin::WallPlugin;
