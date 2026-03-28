//! Dispatches cell-defined effects to the cell entity when spawned.
//!
//! Reads the cell's `CellTypeDefinition.effects` (optional) and pushes
//! children to the cell entity's `BoundEffects`.
//! Stub — real implementation in Wave 6.

use bevy::prelude::*;

/// Dispatches effects from cell definitions to cell entities.
/// Stub — real implementation in Wave 6.
pub(crate) fn dispatch_cell_effects(
    mut _commands: Commands,
) {
    // TODO: Wave 6 — when cells spawn, check CellTypeDefinition.effects,
    // resolve RootEffect targets, push to BoundEffects on cell entity
}
