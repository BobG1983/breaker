//! Cross-domain re-exports for convenient importing.
//!
//! `use crate::prelude::*` gives the most universally used cross-domain types:
//! entity markers, all state types, all cross-domain messages, effect containers
//! and active-effect components, and common resources.
//!
//! Types are added here as they gain cross-domain consumers. When adding a new
//! cross-domain type, add it to the appropriate submodule file and (if used by
//! 3+ domains) to the curated glob below.

pub(crate) mod components;
pub(crate) mod messages;
pub(crate) mod resources;
pub(crate) mod states;

// --- Curated glob: entity markers, effect state, messages, resources, states ---

pub(crate) use components::{
    ActiveDamageBoosts, ActiveSizeBoosts, ActiveSpeedBoosts, AnchorActive, AnchorPlanted, Bolt,
    BoltServing, BoundEffects, Breaker, Cell, EffectNode, NodeScalingFactor, RootEffect,
    StagedEffects, Wall,
};
pub(crate) use messages::*;
pub(crate) use resources::{GameRng, InputActions, PlayfieldConfig};
pub(crate) use states::*;
