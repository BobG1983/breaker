//! Volatile behavior components.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a volatile-type cell.
///
/// Never removed. Identifies cells that participate in the volatile detonation
/// mechanic. The detonation is driven entirely by a `BoundEffects` entry
/// stamped at spawn that carries the damage/radius baked into its
/// `ExplodeConfig`. This marker lets systems query "is this cell volatile?"
/// without walking the `BoundEffects` tree.
#[derive(Component, Debug)]
pub struct VolatileCell;
