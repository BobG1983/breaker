//! Physics domain plugin — CCD collision detection, collision response, wall entities.

pub mod ccd;
pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
pub mod sets;
pub mod systems;

pub use plugin::PhysicsPlugin;
pub use resources::{PhysicsConfig, PhysicsDefaults};
pub use sets::PhysicsSystems;
