//! Terminal — leaf operations in an effect tree.

use serde::{Deserialize, Serialize};

use super::{EffectType, RouteType, ScopedTerminal, Tree};

/// A leaf operation in an effect tree. Either fires an effect directly
/// or routes a tree to another entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terminal {
    /// Execute an effect immediately on the Owner.
    Fire(EffectType),
    /// Install a tree on another entity. `RouteType` controls permanence.
    Route(RouteType, Box<Tree>),
}

/// Widens a `ScopedTerminal` (reversible-only fires) to a `Terminal`
/// (any effect type fires) for armed On entries.
impl From<ScopedTerminal> for Terminal {
    fn from(scoped: ScopedTerminal) -> Self {
        match scoped {
            ScopedTerminal::Fire(reversible) => Self::Fire(EffectType::from(reversible)),
            ScopedTerminal::Route(route_type, tree) => Self::Route(route_type, tree),
        }
    }
}
