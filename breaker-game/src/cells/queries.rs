//! Cells domain query type aliases.
//!
//! Query types with 4+ components live here rather than inline in system files.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::cells::components::{
    CellDamageVisuals, CellHealth, CellHeight, CellWidth, Locked, RequiredToClear,
};

/// Cell entity data needed by bolt-cell collision.
pub(crate) type CollisionQueryCell = (
    Entity,
    &'static Position2D,
    &'static CellWidth,
    &'static CellHeight,
    Option<&'static CellHealth>,
);

/// Cell health, material, damage visuals, clear-requirement, and lock status.
pub(crate) type DamageVisualQuery = (
    &'static mut CellHealth,
    &'static MeshMaterial2d<ColorMaterial>,
    &'static CellDamageVisuals,
    Has<RequiredToClear>,
    Has<Locked>,
);
