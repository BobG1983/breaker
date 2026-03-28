//! Cell-wall collision detection.
//!
//! Detects when a cell entity overlaps a wall entity and sends
//! [`CellImpactWall`] messages. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Cell)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::{components::Cell, messages::CellImpactWall},
    wall::components::Wall,
};

/// Detects cell-wall collisions via AABB overlap.
///
/// For each cell, checks all wall entities for overlap. Sends
/// [`CellImpactWall`] for each detected collision. Currently a
/// placeholder: future moving-cell mechanics will make this active.
pub(crate) fn detect_cell_wall_collision(
    cell_query: Query<(Entity, &Position2D, &Aabb2D), With<Cell>>,
    wall_query: Query<(Entity, &Position2D, &Aabb2D), With<Wall>>,
    mut writer: MessageWriter<CellImpactWall>,
) {
    for (cell_entity, cell_pos, cell_aabb) in &cell_query {
        let cell_half = cell_aabb.half_extents;

        for (wall_entity, wall_pos, wall_aabb) in &wall_query {
            let wall_half = wall_aabb.half_extents;

            // Simple AABB overlap test
            let dx = (cell_pos.0.x - wall_pos.0.x).abs();
            let dy = (cell_pos.0.y - wall_pos.0.y).abs();

            if dx < cell_half.x + wall_half.x && dy < cell_half.y + wall_half.y {
                writer.write(CellImpactWall {
                    cell: cell_entity,
                    wall: wall_entity,
                });
            }
        }
    }
}
