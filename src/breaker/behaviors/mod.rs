//! Breaker archetype behavior system — data-driven trigger→consequence dispatch.

pub mod active;
pub mod bolt_speed_boost;
pub mod bridges;
pub mod definition;
pub mod init;
pub mod life_lost;
mod plugin;
pub mod registry;

pub use definition::ArchetypeDefinition;
pub use plugin::BehaviorPlugin;
pub use registry::ArchetypeRegistry;
