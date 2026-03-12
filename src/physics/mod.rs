//! Physics domain plugin — CCD collision detection, collision response, wall entities.

pub mod ccd;
pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
pub use plugin::PhysicsPlugin;
pub use resources::{PhysicsConfig, PhysicsDefaults};

/// System sets exported by the physics domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystems {
    /// The `bolt_breaker_collision` system — detects and resolves bolt-breaker hits.
    BreakerCollision,
}
