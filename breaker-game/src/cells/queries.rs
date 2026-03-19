//! Cells domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::prelude::*;

use crate::cells::components::{CellDamageVisuals, CellHealth, RequiredToClear};

/// Cell health, material, damage visuals, and clear-requirement for damage feedback.
pub(crate) type CellDamageVisualQuery = (
    &'static mut CellHealth,
    &'static MeshMaterial2d<ColorMaterial>,
    &'static CellDamageVisuals,
    Has<RequiredToClear>,
);
