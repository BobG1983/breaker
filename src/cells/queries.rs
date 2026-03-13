//! Cells domain query type aliases — clippy `type_complexity` lint.

use bevy::prelude::*;

use crate::cells::components::{CellDamageVisuals, CellHealth, RequiredToClear};

/// Cell health, material, damage visuals, and clear-requirement for damage feedback.
pub type CellDamageVisualQuery = (
    &'static mut CellHealth,
    &'static MeshMaterial2d<ColorMaterial>,
    &'static CellDamageVisuals,
    Has<RequiredToClear>,
);
