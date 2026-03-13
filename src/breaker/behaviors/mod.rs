//! Breaker archetype behavior system — data-driven trigger→consequence dispatch.

pub mod active;
pub mod bridges;
pub mod consequences;
pub mod definition;
pub mod init;
mod plugin;
pub mod registry;

pub use definition::ArchetypeDefinition;
pub use plugin::BehaviorPlugin;
pub use registry::ArchetypeRegistry;
