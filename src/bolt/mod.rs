//! Bolt domain plugin — bolt physics, reflection model, speed management.

pub mod components;
pub mod filters;
mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
pub use plugin::BoltPlugin;
pub use resources::{BoltConfig, BoltDefaults};

/// System sets exported by the bolt domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoltSystems {
    /// The `prepare_bolt_velocity` system — copies bolt velocity for physics.
    PrepareVelocity,
}
