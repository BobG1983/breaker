//! Bolt-wall overlap collision detection.
//!
//! After `bolt_cell_collision` moves the bolt via CCD against cells, this
//! system checks whether the bolt ended up overlapping any wall and resolves
//! it. Uses `query_aabb_filtered` on the `CollisionQuadtree` for broad-phase
//! detection, then verifies actual AABB overlap expanded by the bolt radius.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, prelude::reflect, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{
        components::Bolt, filters::ActiveFilter, messages::BoltImpactWall,
        queries::CollisionQueryBolt,
    },
    shared::WALL_LAYER,
    wall::components::Wall,
};

/// Wall entity lookup for overlap detection — avoids clippy `type_complexity`.
type WallLookup<'w, 's> =
    Query<'w, 's, (&'static Position2D, &'static Aabb2D), (With<Wall>, Without<Bolt>)>;

/// Detects bolt-wall overlaps and resolves them via push-out and velocity reflection.
///
/// For each active bolt, queries the quadtree for walls within the bolt's radius.
/// If a wall overlap is confirmed, the bolt is pushed out to a safe position,
/// its velocity is reflected off the nearest wall face, and `PiercingRemaining`
/// is reset to `EffectivePiercing.0`.
pub(crate) fn bolt_wall_collision(
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    let query_layers = CollisionLayers::new(0, WALL_LAYER);

    for (
        bolt_entity,
        mut bolt_position,
        mut bolt_vel,
        _,
        bolt_radius,
        mut piercing_remaining,
        effective_piercing,
        _,
        bolt_entity_scale,
        _,
    ) in &mut bolt_query
    {
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let position = bolt_position.0;
        let velocity = bolt_vel.0;

        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&Aabb2D::new(position, Vec2::splat(r)), query_layers);

        for wall_entity in candidates {
            let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
                continue;
            };

            // Compute expanded AABB (wall AABB in world space, expanded by bolt radius)
            let wall_center = wall_pos.0 + wall_aabb.center;
            let expanded_half = wall_aabb.half_extents + Vec2::splat(r);

            // Strict inequality — tangent (exactly on edge) does not count as overlap
            let inside = position.x > wall_center.x - expanded_half.x
                && position.x < wall_center.x + expanded_half.x
                && position.y > wall_center.y - expanded_half.y
                && position.y < wall_center.y + expanded_half.y;

            if !inside {
                continue;
            }

            // Find the nearest face for push-out direction
            let dist_left = (position.x - (wall_center.x - expanded_half.x)).abs();
            let dist_right = (position.x - (wall_center.x + expanded_half.x)).abs();
            let dist_bottom = (position.y - (wall_center.y - expanded_half.y)).abs();
            let dist_top = (position.y - (wall_center.y + expanded_half.y)).abs();

            let faces = [
                (
                    dist_left,
                    Vec2::NEG_X,
                    Vec2::new(wall_center.x - expanded_half.x, position.y),
                ),
                (
                    dist_right,
                    Vec2::X,
                    Vec2::new(wall_center.x + expanded_half.x, position.y),
                ),
                (
                    dist_bottom,
                    Vec2::NEG_Y,
                    Vec2::new(position.x, wall_center.y - expanded_half.y),
                ),
                (
                    dist_top,
                    Vec2::Y,
                    Vec2::new(position.x, wall_center.y + expanded_half.y),
                ),
            ];
            let Some((_, normal, push_pos)) = faces
                .into_iter()
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            else {
                continue;
            };

            // Push bolt to the nearest face and reflect velocity
            bolt_position.0 = push_pos;
            bolt_vel.0 = reflect(velocity, normal);

            // Reset PiercingRemaining to EffectivePiercing.0
            if let (Some(pr), Some(ep)) = (&mut piercing_remaining, effective_piercing) {
                pr.0 = ep.0;
            }

            writer.write(BoltImpactWall {
                bolt: bolt_entity,
                wall: wall_entity,
            });

            // Only resolve the first wall overlap per bolt per frame
            break;
        }
    }
}
