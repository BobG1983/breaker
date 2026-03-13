//! Physics domain plugin — CCD collision detection and collision response.

pub mod ccd;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod sets;
pub mod systems;

pub use plugin::PhysicsPlugin;
pub use sets::PhysicsSystems;
