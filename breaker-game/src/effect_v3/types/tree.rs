//! Tree — the recursive effect tree enum.

use serde::{Deserialize, Serialize};

use super::{Condition, EffectType, ParticipantTarget, ScopedTree, Terminal, Trigger};

/// The recursive effect tree. Each variant represents a different
/// control-flow node that gates or sequences effect execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tree {
    /// Execute an effect immediately on the Owner.
    Fire(EffectType),
    /// Repeating gate. Every time the trigger matches, evaluate the inner tree.
    When(Trigger, Box<Self>),
    /// One-shot gate. Evaluates inner tree on first trigger match, then removes itself.
    Once(Trigger, Box<Self>),
    /// State-scoped. Applies inner effects while a condition is true, reverses them when false.
    During(Condition, Box<ScopedTree>),
    /// Event-scoped. Applies inner effects immediately, reverses them when the trigger fires.
    Until(Trigger, Box<ScopedTree>),
    /// Ordered multi-execute. Runs children left to right.
    Sequence(Vec<Terminal>),
    /// Redirects a terminal to a different entity involved in the trigger event.
    On(ParticipantTarget, Terminal),
}
