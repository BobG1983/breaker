//! Breaker-cell collision detection.
//!
//! Detects when the breaker entity overlaps a cell entity and sends
//! [`BreakerImpactCell`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Cell)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{Breaker, BreakerHeight, BreakerWidth},
        messages::BreakerImpactCell,
    },
    cells::components::Cell,
    shared::{BREAKER_LAYER, CELL_LAYER, EntityScale},
};

/// Breaker query data for cell collision detection.
type BreakerCellCollisionQuery = (
    Entity,
    &'static Position2D,
    &'static BreakerWidth,
    &'static BreakerHeight,
    Option<&'static EntityScale>,
);

/// Cell entity lookup for narrow-phase overlap verification.
type CellLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Cell>>;

/// Detects breaker-cell collisions via quadtree AABB query.
///
/// For each breaker, queries the quadtree for nearby cell entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`BreakerImpactCell`].
pub(crate) fn breaker_cell_collision(
    quadtree: Res<CollisionQuadtree>,
    breaker_query: Query<BreakerCellCollisionQuery, With<Breaker>>,
    cell_lookup: CellLookup,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    let Ok((breaker_entity, breaker_pos, breaker_w, breaker_h, breaker_scale)) =
        breaker_query.single()
    else {
        return;
    };

    let scale = breaker_scale.map_or(1.0, |s| s.0);
    let half_w = breaker_w.half_width() * scale;
    let half_h = breaker_h.half_height() * scale;

    let breaker_aabb = Aabb2D::new(breaker_pos.0, Vec2::new(half_w, half_h));
    let layers = CollisionLayers::new(BREAKER_LAYER, CELL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for cell_entity in candidates {
        let Ok((cell_pos, cell_aabb)) = cell_lookup.get(cell_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker_pos.0.x - cell_pos.0.x).abs();
        let dy = (breaker_pos.0.y - cell_pos.0.y).abs();
        if dx < half_w + cell_aabb.half_extents.x && dy < half_h + cell_aabb.half_extents.y {
            writer.write(BreakerImpactCell {
                breaker: breaker_entity,
                cell: cell_entity,
            });
        }
    }
}
