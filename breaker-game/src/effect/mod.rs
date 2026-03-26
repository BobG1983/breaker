//! Effect system — data-driven trigger->effect dispatch.

pub(crate) mod active;
pub(crate) mod armed;
pub(crate) mod bridges;
pub(crate) mod definition;
pub(crate) mod effect_nodes;
pub(crate) mod effects;
pub(crate) mod evaluate;
pub(crate) mod helpers;
pub(crate) mod init;
mod plugin;
pub(crate) mod registry;
pub(crate) mod sets;
pub(crate) mod triggers;
pub(crate) mod typed_events;

pub use active::ActiveEffects;
pub(crate) use definition::BreakerDefinition;
pub use definition::{
    AttractionType, Effect, EffectChains, EffectEntity, EffectNode, EffectTarget, ImpactTarget,
    RootEffect, Target, Trigger,
};
pub(crate) use plugin::EffectPlugin;
pub(crate) use registry::BreakerRegistry;
pub(crate) use sets::EffectSystems;
