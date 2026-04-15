//! Bolt-wall overlap collision detection.
//!
//! After `bolt_cell_collision` moves the bolt via CCD against cells, this
//! system checks whether the bolt ended up overlapping any wall and resolves
//! it. Uses `query_aabb_filtered` on the `CollisionQuadtree` for broad-phase
//! detection, then verifies actual AABB overlap expanded by the bolt radius.

use bevy::prelude::*;
use rantzsoft_physics2d::{prelude::reflect, resources::CollisionQuadtree};

use crate::{
    bolt::{
        components::{LastImpact, wall_normal_to_impact_side},
        filters::ActiveFilter,
        queries::{BoltCollisionData, apply_velocity_formula},
    },
    effect_v3::stacking::EffectStack,
    prelude::*,
};

/// Wall entity lookup for overlap detection — avoids clippy `type_complexity`.
type WallLookup<'w, 's> =
    Query<'w, 's, (&'static Position2D, &'static Aabb2D), (With<Wall>, Without<Bolt>)>;

/// Detects bolt-wall overlaps and resolves them via push-out and velocity reflection.
///
/// For each active bolt, queries the quadtree for walls within the bolt's radius.
/// If a wall overlap is confirmed, the bolt is pushed out to a safe position,
/// its velocity is reflected off the nearest wall face, and `PiercingRemaining`
/// is reset to `EffectStack<PiercingConfig>::aggregate()`.
pub(crate) fn bolt_wall_collision(
    mut commands: Commands,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<BoltCollisionData, ActiveFilter>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    let query_layers = CollisionLayers::new(0, WALL_LAYER);

    for mut bolt in &mut bolt_query {
        let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
        let r = bolt.collision.radius.0 * bolt_scale;
        let position = bolt.spatial.position.0;
        let velocity = bolt.spatial.velocity.0;

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
            bolt.spatial.position.0 = push_pos;
            bolt.spatial.velocity.0 = reflect(velocity, normal);

            // Apply the canonical velocity formula after reflection
            apply_velocity_formula(
                &mut bolt.spatial,
                bolt.collision
                    .active_speed_boosts
                    .map_or(1.0, EffectStack::aggregate),
            );

            // Stamp LastImpact at the push-out position with the side
            // derived from the wall push-out normal (inverted mapping).
            let side = wall_normal_to_impact_side(normal);
            if let Some(li) = bolt.collision.last_impact.as_mut() {
                li.position = push_pos;
                li.side = side;
            } else {
                commands.entity(bolt.entity).insert(LastImpact {
                    position: push_pos,
                    side,
                });
            }

            // Reset PiercingRemaining to EffectStack<PiercingConfig>::aggregate()
            if let (Some(pr), Some(ap)) = (
                &mut bolt.collision.piercing_remaining,
                bolt.collision.active_piercings,
            ) {
                pr.0 = ap.aggregate().round().max(0.0) as u32;
            }

            writer.write(BoltImpactWall {
                bolt: bolt.entity,
                wall: wall_entity,
            });

            // Only resolve the first wall overlap per bolt per frame
            break;
        }
    }
}
