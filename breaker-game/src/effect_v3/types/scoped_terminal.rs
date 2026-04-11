//! `ScopedTerminal` — leaf operations inside During/Until scoped trees.

use serde::{Deserialize, Serialize};

use super::{ReversibleEffectType, RouteType, Tree};

/// A leaf operation inside a During/Until scoped tree.
/// Fire variants are restricted to reversible effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopedTerminal {
    /// Execute a reversible effect immediately on the Owner.
    Fire(ReversibleEffectType),
    /// Install a tree on another entity. `RouteType` controls permanence.
    Route(RouteType, Box<Tree>),
}
