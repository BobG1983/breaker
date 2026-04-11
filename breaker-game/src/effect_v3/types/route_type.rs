//! `RouteType` — controls how a tree is installed on a target entity.

use serde::{Deserialize, Serialize};

/// Controls whether a routed tree is permanent or one-shot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RouteType {
    /// Permanently install the tree into the target's `BoundEffects`.
    /// The tree re-arms after each trigger match.
    Bound,
    /// Install the tree as a one-shot into the target's `StagedEffects`.
    /// Consumed after one trigger match.
    Staged,
}
