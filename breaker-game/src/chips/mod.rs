//! Chips domain plugin — passive and triggered chip effects.

pub(crate) mod components;
pub mod definition;
pub(crate) mod effects;
pub mod inventory;
pub(crate) mod offering;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use definition::{ChipDefinition, ChipTemplate, expand_template};
pub use definition::{ImpactTarget, Target, TriggerChain};
pub(crate) use plugin::ChipsPlugin;
pub(crate) use resources::{ChipRegistry, EvolutionRegistry};
