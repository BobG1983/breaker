//! Cells domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::prelude::*;

use crate::cells::components::{CellDamageVisuals, CellHealth, Locked, RequiredToClear};

/// Cell health, material, damage visuals, clear-requirement, and lock status.
pub(crate) type DamageVisualQuery = (
    &'static mut CellHealth,
    &'static MeshMaterial2d<ColorMaterial>,
    &'static CellDamageVisuals,
    Has<RequiredToClear>,
    Has<Locked>,
);
