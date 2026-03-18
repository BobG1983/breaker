//! Physics domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the physics domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PhysicsSystems {
    /// The `bolt_breaker_collision` system — detects and resolves bolt-breaker hits.
    BreakerCollision,
    /// The `bolt_lost` system — detects bolt below playfield and respawns.
    BoltLost,
}
