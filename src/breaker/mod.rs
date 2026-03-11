//! Breaker domain plugin — breaker mechanics, state machine, bump system.

pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
pub mod systems;

use bevy::prelude::*;
pub use plugin::BreakerPlugin;
pub use resources::{BreakerConfig, BreakerDefaults};

/// System sets exported by the breaker domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    /// The `move_breaker` system — updates breaker position from input.
    Move,
}
