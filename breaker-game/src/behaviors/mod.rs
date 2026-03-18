//! Behavior system — data-driven trigger→consequence dispatch.

pub(crate) mod active;
pub(crate) mod bridges;
pub(crate) mod consequences;
pub(crate) mod definition;
pub(crate) mod init;
mod plugin;
pub(crate) mod registry;
pub(crate) mod sets;

pub(crate) use definition::ArchetypeDefinition;
pub(crate) use plugin::BehaviorsPlugin;
pub(crate) use registry::ArchetypeRegistry;
pub(crate) use sets::BehaviorSystems;
