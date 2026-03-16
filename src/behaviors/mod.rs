//! Behavior system — data-driven trigger→consequence dispatch.

pub mod active;
pub mod bridges;
pub mod consequences;
pub mod definition;
pub mod init;
mod plugin;
pub mod registry;
pub mod sets;

pub use definition::ArchetypeDefinition;
pub use plugin::BehaviorsPlugin;
pub use registry::ArchetypeRegistry;
pub use sets::BehaviorSystems;
