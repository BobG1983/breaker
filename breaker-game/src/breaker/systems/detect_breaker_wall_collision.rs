//! Breaker-wall collision detection.
//!
//! Detects when the breaker entity overlaps a wall entity and sends
//! [`BreakerImpactWall`] messages. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{Breaker, BreakerHeight, BreakerWidth},
        messages::BreakerImpactWall,
    },
    shared::EntityScale,
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

/// Detects breaker-wall collisions via AABB overlap.
///
/// For each breaker, checks all wall entities for overlap. Sends
/// [`BreakerImpactWall`] for each detected collision. The breaker
/// already clamps to playfield bounds in `move_breaker`, so this
/// detects edge-case overlaps for effect trigger chains.
pub(crate) fn detect_breaker_wall_collision(
    breaker_query: Query<BreakerWallCollisionQuery, With<Breaker>>,
    wall_query: Query<(Entity, &Position2D, &Aabb2D), With<Wall>>,
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

    for (wall_entity, wall_pos, wall_aabb) in &wall_query {
        let wall_half = wall_aabb.half_extents;

        // Simple AABB overlap test
        let dx = (breaker_pos.0.x - wall_pos.0.x).abs();
        let dy = (breaker_pos.0.y - wall_pos.0.y).abs();

        if dx < half_w + wall_half.x && dy < half_h + wall_half.y {
            writer.write(BreakerImpactWall {
                breaker: breaker_entity,
                wall: wall_entity,
            });
        }
    }
}
