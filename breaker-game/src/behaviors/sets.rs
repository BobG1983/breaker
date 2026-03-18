//! Behavior system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the behaviors domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BehaviorSystems {
    /// Behavior bridge systems — translate messages into consequence events.
    ///
    /// Observers fire synchronously during bridge execution, so messages
    /// written by consequence handlers are available to downstream systems
    /// that order `.after(BehaviorSystems::Bridge)`.
    Bridge,
}
