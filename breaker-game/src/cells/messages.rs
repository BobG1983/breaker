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
