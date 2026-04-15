//! Cross-domain re-exports for convenient importing.
//!
//! `use crate::prelude::*` gives the most universally used cross-domain types:
//! entity markers, all state types, all cross-domain messages, effect containers
//! and active-effect components, common resources, death pipeline types, collision
//! layer constants, and (in test builds) shared test infrastructure.
//!
//! Types are added here as they gain 3+ consumer files anywhere in the codebase.
//! When adding a new type, place it in the appropriate submodule file — it will
//! be picked up automatically by the submodule glob re-exports below.

// Prelude re-exports are consumed via wildcard imports (`use crate::prelude::*`)
// which clippy cannot trace — individual items appear unused even when they aren't.
#![allow(unused_imports, reason = "prelude items consumed via wildcard imports")]

pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod death_pipeline;
pub(crate) mod messages;
pub(crate) mod resources;
pub(crate) mod states;

#[cfg(test)]
pub(crate) mod test_utils;

pub(crate) use components::*;
pub(crate) use constants::*;
pub(crate) use death_pipeline::*;
pub(crate) use messages::*;
pub(crate) use resources::*;
pub(crate) use states::*;
#[cfg(test)]
pub(crate) use test_utils::*;
