//! Breaker-wall collision detection.
//!
//! Detects when the breaker entity overlaps a wall entity and sends
//! [`BreakerImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{Breaker, BreakerHeight, BreakerWidth},
        messages::BreakerImpactWall,
    },
    shared::{BREAKER_LAYER, EntityScale, WALL_LAYER},
    wall::components::Wall,
};

/// Breaker query data for wall collision detection.
type BreakerWallCollisionQuery = (
    Entity,
    &'static Position2D,
    &'static BreakerWidth,
    &'static BreakerHeight,
    Option<&'static EntityScale>,
);

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
    breaker_query: Query<BreakerWallCollisionQuery, With<Breaker>>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<BreakerImpactWall>,
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
    let layers = CollisionLayers::new(BREAKER_LAYER, WALL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for wall_entity in candidates {
        let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker_pos.0.x - wall_pos.0.x).abs();
        let dy = (breaker_pos.0.y - wall_pos.0.y).abs();
        if dx < half_w + wall_aabb.half_extents.x && dy < half_h + wall_aabb.half_extents.y {
            writer.write(BreakerImpactWall {
                breaker: breaker_entity,
                wall: wall_entity,
            });
        }
    }
}
