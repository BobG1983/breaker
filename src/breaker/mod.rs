//! Breaker domain plugin — breaker mechanics, state machine, bump system.

pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
pub mod systems;

pub use plugin::BreakerPlugin;
pub use resources::BreakerConfig;
