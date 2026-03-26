//! Breaker domain plugin — breaker mechanics, state machine, bump system.

pub mod components;
pub(crate) mod definition;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub(crate) mod registry;
pub mod resources;
pub mod sets;
pub mod systems;

pub(crate) use definition::BreakerDefinition;
pub use plugin::BreakerPlugin;
pub(crate) use registry::BreakerRegistry;
pub use resources::{BreakerConfig, BreakerDefaults, ForceBumpGrade};
pub use sets::BreakerSystems;
