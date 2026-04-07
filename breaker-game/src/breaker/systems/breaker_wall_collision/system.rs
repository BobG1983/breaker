//! Breaker-wall collision detection.
//!
//! Detects when the breaker entity overlaps a wall entity and sends
//! [`BreakerImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use crate::{
    breaker::queries::BreakerSizeData,
    prelude::*,
    shared::{BREAKER_LAYER, WALL_LAYER},
};

/// Wall entity lookup for narrow-phase overlap verification.
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;

/// Detects breaker-wall collisions via quadtree AABB query.
///
/// For each breaker, queries the quadtree for nearby wall entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`BreakerImpactWall`]. The breaker already
/// clamps to playfield bounds in `move_breaker`, so this detects
/// edge-case overlaps for effect trigger chains.
pub(crate) fn breaker_wall_collision(
    quadtree: Res<CollisionQuadtree>,
    breaker_query: Query<BreakerSizeData, With<Breaker>>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<BreakerImpactWall>,
) {
    let Ok(breaker) = breaker_query.single() else {
        return;
    };

    let size_mult = breaker
        .size_boosts
        .map_or(1.0, ActiveSizeBoosts::multiplier);
    let scale = breaker.node_scale.map_or(1.0, |s| s.0);
    let half_w = breaker.base_width.half_width() * size_mult * scale;
    let half_h = breaker.base_height.half_height() * size_mult * scale;

    let breaker_aabb = Aabb2D::new(breaker.position.0, Vec2::new(half_w, half_h));
    let layers = CollisionLayers::new(BREAKER_LAYER, WALL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for wall_entity in candidates {
        let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker.position.0.x - wall_pos.0.x).abs();
        let dy = (breaker.position.0.y - wall_pos.0.y).abs();
        if dx < half_w + wall_aabb.half_extents.x && dy < half_h + wall_aabb.half_extents.y {
            writer.write(BreakerImpactWall {
                breaker: breaker.entity,
                wall: wall_entity,
            });
        }
    }
}
