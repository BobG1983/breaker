//! Cells domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::components::{CellDamageVisuals, CellHealth, Locked, RequiredToClear},
    effect::effects::shield::ShieldActive,
};

/// Cell health, material, damage visuals, clear-requirement, lock status, position, and shield status.
pub(crate) type DamageVisualQuery = (
    &'static mut CellHealth,
    &'static MeshMaterial2d<ColorMaterial>,
    &'static CellDamageVisuals,
    Has<RequiredToClear>,
    Has<Locked>,
    &'static Position2D,
    Option<&'static mut ShieldActive>,
);
