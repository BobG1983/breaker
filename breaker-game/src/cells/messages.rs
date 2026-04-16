//! Messages sent by the cells domain.

use bevy::prelude::*;

/// Sent when a cell collides with a wall.
///
/// Consumed by `bridge_wall_impact` and `bridge_cell_impacted` in the effect domain.
/// Relevant for future moving-cell mechanics.
#[derive(Message, Clone, Debug)]
pub(crate) struct CellImpactWall {
    /// The cell entity that collided with the wall.
    pub cell: Entity,
    /// The wall entity that was hit.
    pub wall: Entity,
}

/// Sent when a bolt hits a portal cell.
///
/// Consumed by `handle_portal_entered` (mock — immediately produces `PortalCompleted`).
#[derive(Message, Clone, Debug)]
pub(crate) struct PortalEntered {
    pub(crate) portal: Entity,
    #[cfg(test)]
    pub(crate) bolt:   Entity,
}

/// Sent when a portal's sub-node is completed (or mock-completed).
///
/// Consumed by `handle_portal_completed` which kills the portal cell.
#[derive(Message, Clone, Debug)]
pub(crate) struct PortalCompleted {
    pub(crate) portal: Entity,
}

/// Sent when a Salvo hits the Breaker entity.
///
/// Consumed by effect bridges in Wave 6C.
#[derive(Message, Clone, Debug)]
pub(crate) struct SalvoImpactBreaker {
    /// The salvo entity that hit the breaker.
    pub salvo:   Entity,
    /// The breaker entity that was hit.
    pub breaker: Entity,
}
