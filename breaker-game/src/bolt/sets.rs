//! Bolt domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the bolt domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoltSystems {
    /// The `reset_bolt` system — resets bolt position and velocity at node start.
    Reset,
    /// The `bolt_cell_collision` system — detects and resolves bolt-cell hits.
    CellCollision,
    /// The `bolt_wall_collision` system — detects and resolves bolt-wall hits.
    WallCollision,
    /// The `bolt_breaker_collision` system — detects and resolves bolt-breaker hits.
    BreakerCollision,
    /// The `bolt_lost` system — detects bolt below playfield and respawns.
    BoltLost,
}
