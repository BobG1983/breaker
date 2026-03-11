//! Physics domain plugin — quadtree, collision detection, collision response.

pub mod messages;
mod plugin;
pub mod resources;
mod systems;

pub use plugin::PhysicsPlugin;
pub use resources::PhysicsConfig;
