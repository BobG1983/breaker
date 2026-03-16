//! Breaker domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the breaker domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    /// The `move_breaker` system — updates breaker position from input.
    Move,
    /// Behavior bridge systems — translate messages into consequence events.
    ///
    /// Observers fire synchronously during bridge execution, so messages
    /// written by consequence handlers are available to downstream systems
    /// that order `.after(BreakerSystems::BehaviorBridge)`.
    BehaviorBridge,
}
