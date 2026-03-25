//! Effect system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the effect domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum EffectSystems {
    /// Effect bridge systems — translate messages into consequence events.
    ///
    /// Observers fire synchronously during bridge execution, so messages
    /// written by consequence handlers are available to downstream systems
    /// that order `.after(EffectSystems::Bridge)`.
    Bridge,
}
