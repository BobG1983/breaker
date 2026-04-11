//! `RootNode` — top-level entry point for effect definitions.

use serde::{Deserialize, Serialize};

use super::{EntityKind, StampTarget, Tree};

/// Top-level entry point for effect definitions in RON.
/// Either stamps a tree onto existing entities or watches for new spawns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootNode {
    /// Install a tree on a target entity or entity group.
    Stamp(StampTarget, Tree),
    /// Watch for new entities of a given kind and apply a tree to each one.
    Spawn(EntityKind, Tree),
}
