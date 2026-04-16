//! Magnetic behavior components.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a magnetic-type cell.
///
/// Never removed. Inserted alongside `MagneticField` when
/// `CellBehavior::Magnetic` is resolved at spawn time.
#[derive(Component, Debug)]
pub struct MagneticCell;

/// Configuration for a magnetic cell's attraction field.
///
/// - `radius`: maximum distance at which the field affects bolts.
/// - `strength`: force coefficient for the inverse-square attraction.
#[derive(Component, Debug, Clone, Copy)]
pub struct MagneticField {
    /// Maximum attraction radius in world units.
    pub radius:   f32,
    /// Force strength coefficient for inverse-square attraction.
    pub strength: f32,
}
