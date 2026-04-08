//! Cross-domain re-exports for convenient importing.
//!
//! `use crate::prelude::*` gives the most universally used cross-domain types:
//! entity markers, all state types, all cross-domain messages, effect containers
//! and active-effect components, and common resources.
//!
//! Types are added here as they gain cross-domain consumers. When adding a new
//! cross-domain type, add it to the appropriate submodule file and (if used by
//! 3+ domains) to the curated glob below.

// Prelude re-exports are consumed via wildcard imports (`use crate::prelude::*`)
// which clippy cannot trace — individual items appear unused even when they aren't.
#![allow(unused_imports, reason = "prelude items consumed via wildcard imports")]

pub(crate) mod components;
pub(crate) mod messages;
pub(crate) mod resources;
pub(crate) mod states;

pub(crate) use components::*;
pub(crate) use messages::*;
pub(crate) use resources::*;
pub(crate) use states::*;
