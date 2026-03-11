//! Physics domain components.

use bevy::prelude::*;

/// Marker component identifying wall entities (left, right, ceiling).
#[derive(Component, Debug)]
pub struct Wall;

/// Half-extents for a wall entity used in CCD collision.
///
/// Walls are invisible collision boundaries, so they carry their own size
/// rather than relying on cell config.
#[derive(Component, Debug)]
pub struct WallSize {
    /// Half-width in world units.
    pub half_width: f32,
    /// Half-height in world units.
    pub half_height: f32,
}
