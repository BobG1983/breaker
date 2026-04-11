//! `ScopedTree` — restricted tree inside During/Until scoped contexts.

use serde::{Deserialize, Serialize};

use super::{ParticipantTarget, ReversibleEffectType, ScopedTerminal, Tree, Trigger};

/// A restricted tree that appears inside During/Until scoped contexts.
/// Fire variants are limited to reversible effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopedTree {
    /// Execute a reversible effect immediately on the Owner.
    Fire(ReversibleEffectType),
    /// Ordered multi-execute of reversible effects. All children must be reversible.
    Sequence(Vec<ReversibleEffectType>),
    /// Repeating gate inside a scoped context. Inner tree is unrestricted.
    When(Trigger, Box<Tree>),
    /// Redirects a scoped terminal to a different entity involved in the trigger event.
    On(ParticipantTarget, ScopedTerminal),
}
