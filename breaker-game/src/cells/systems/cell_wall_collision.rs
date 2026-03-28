//! Cell-wall collision detection.
//!
//! Detects when a cell entity overlaps a wall entity and sends
//! [`CellImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Cell)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::{components::Cell, messages::CellImpactWall},
    shared::{CELL_LAYER, WALL_LAYER},
    wall::components::Wall,
};

/// Wall entity lookup for narrow-phase overlap verification.
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;

/// Detects cell-wall collisions via quadtree AABB query.
///
/// For each cell, queries the quadtree for nearby wall entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`CellImpactWall`].
pub(crate) fn cell_wall_collision(
    quadtree: Res<CollisionQuadtree>,
    cell_query: Query<(Entity, &Position2D, &Aabb2D), With<Cell>>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<CellImpactWall>,
) {
    let layers = CollisionLayers::new(CELL_LAYER, WALL_LAYER);

    for (cell_entity, cell_pos, cell_aabb) in &cell_query {
        let cell_aabb_query = Aabb2D::new(cell_pos.0, cell_aabb.half_extents);
        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&cell_aabb_query, layers);

        for wall_entity in candidates {
            let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
                continue;
            };

            // Narrow-phase: verify actual AABB overlap
            let dx = (cell_pos.0.x - wall_pos.0.x).abs();
            let dy = (cell_pos.0.y - wall_pos.0.y).abs();
            if dx < cell_aabb.half_extents.x + wall_aabb.half_extents.x
                && dy < cell_aabb.half_extents.y + wall_aabb.half_extents.y
            {
                writer.write(CellImpactWall {
                    cell: cell_entity,
                    wall: wall_entity,
                });
            }
        }
    }
}
