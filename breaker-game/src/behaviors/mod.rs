//! Behavior system — data-driven trigger→effect dispatch.

pub(crate) mod active;
pub(crate) mod armed;
pub(crate) mod bridges;
pub(crate) mod definition;
pub(crate) mod effects;
pub(crate) mod evaluate;
pub(crate) mod events;
pub(crate) mod init;
mod plugin;
pub(crate) mod registry;
pub(crate) mod sets;

pub use active::ActiveChains;
pub(crate) use definition::ArchetypeDefinition;
pub(crate) use plugin::BehaviorsPlugin;
pub(crate) use registry::ArchetypeRegistry;
pub(crate) use sets::BehaviorSystems;
