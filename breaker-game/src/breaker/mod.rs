//! Breaker domain plugin — breaker mechanics, state machine, bump system.

pub(crate) mod builder;
pub mod components;
pub mod definition;
pub mod filters;
pub mod messages;
mod plugin;
pub mod queries;
pub mod registry;
pub mod resources;
pub mod sets;
pub mod systems;

pub use definition::BreakerDefinition;
pub use plugin::BreakerPlugin;
pub use registry::BreakerRegistry;
pub use resources::{ForceBumpGrade, SelectedBreaker};
pub use sets::BreakerSystems;
