//! Physics domain plugin — CCD collision detection, collision response, wall entities.

pub mod ccd;
pub mod components;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod sets;
pub mod systems;

pub use plugin::PhysicsPlugin;
pub use sets::PhysicsSystems;
