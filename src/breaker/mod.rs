//! Breaker domain plugin — breaker mechanics, state machine, bump system.

pub mod behaviors;
pub mod components;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod resources;
pub mod sets;
pub mod systems;

pub use plugin::BreakerPlugin;
pub use resources::{BreakerConfig, BreakerDefaults};
pub use sets::BreakerSystems;
