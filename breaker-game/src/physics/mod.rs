//! Physics domain plugin — CCD collision detection and collision response.

pub(crate) mod filters;
pub(crate) mod messages;
mod plugin;
pub(crate) mod queries;
pub(crate) mod sets;
pub(crate) mod systems;

pub(crate) use plugin::PhysicsPlugin;
pub use sets::PhysicsSystems;
