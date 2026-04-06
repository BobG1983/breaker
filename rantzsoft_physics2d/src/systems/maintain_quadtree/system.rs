//! System that synchronizes `Aabb2D` entities with the `CollisionQuadtree`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::GlobalPosition2D;

use crate::{aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree};

/// Keeps the `CollisionQuadtree` in sync with entity spatial data.
///
/// Processing order:
/// 1. Removals (`RemovedComponents<Aabb2D>`)
/// 2. Additions (`Added<Aabb2D>`)
/// 3. Changes (`Changed<GlobalPosition2D>`, `Changed<CollisionLayers>`) — skipping
///    entities that were just added this frame to avoid double-insert.
pub(crate) fn maintain_quadtree(
    mut quadtree: ResMut<CollisionQuadtree>,
    mut removed: RemovedComponents<Aabb2D>,
    added: Query<(Entity, &Aabb2D, &GlobalPosition2D, &CollisionLayers), Added<Aabb2D>>,
    changed_pos: Query<
        (Entity, Ref<Aabb2D>, &GlobalPosition2D, &CollisionLayers),
        Changed<GlobalPosition2D>,
    >,
    changed_layers: Query<
        (Entity, Ref<Aabb2D>, &GlobalPosition2D, &CollisionLayers),
        Changed<CollisionLayers>,
    >,
) {
    // 1. Removals first
    for entity in removed.read() {
        quadtree.quadtree.remove(entity);
    }

    // 2. Additions — compute world-space AABB and insert
    for (entity, aabb, global_pos, layers) in &added {
        let world_aabb = Aabb2D::new(global_pos.0, aabb.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }

    // 3. Changes — global position changed (skip entities just added this frame)
    for (entity, aabb_ref, global_pos, layers) in &changed_pos {
        if aabb_ref.is_added() {
            continue;
        }
        quadtree.quadtree.remove(entity);
        let world_aabb = Aabb2D::new(global_pos.0, aabb_ref.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }

    // 4. Changes — layers changed (skip entities just added or already updated by position change)
    for (entity, aabb_ref, global_pos, layers) in &changed_layers {
        if aabb_ref.is_added() {
            continue;
        }
        // Only update if global position did NOT also change (to avoid double-update).
        // If global position also changed, the position loop already handled the remove+insert.
        if changed_pos.get(entity).is_ok() {
            continue;
        }
        quadtree.quadtree.remove(entity);
        let world_aabb = Aabb2D::new(global_pos.0, aabb_ref.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }
}
