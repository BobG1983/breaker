//! System that synchronizes `Aabb2D` entities with the `CollisionQuadtree`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree};

/// Keeps the `CollisionQuadtree` in sync with entity spatial data.
///
/// Processing order:
/// 1. Removals (`RemovedComponents<Aabb2D>`)
/// 2. Additions (`Added<Aabb2D>`)
/// 3. Changes (`Changed<Position2D>`, `Changed<CollisionLayers>`) — skipping
///    entities that were just added this frame to avoid double-insert.
pub fn maintain_quadtree(
    mut quadtree: ResMut<CollisionQuadtree>,
    mut removed: RemovedComponents<Aabb2D>,
    added: Query<(Entity, &Aabb2D, &Position2D, &CollisionLayers), Added<Aabb2D>>,
    changed_pos: Query<(Entity, Ref<Aabb2D>, &Position2D, &CollisionLayers), Changed<Position2D>>,
    changed_layers: Query<
        (Entity, Ref<Aabb2D>, &Position2D, &CollisionLayers),
        Changed<CollisionLayers>,
    >,
) {
    // 1. Removals first
    for entity in removed.read() {
        quadtree.quadtree.remove(entity);
    }

    // 2. Additions — compute world-space AABB and insert
    for (entity, aabb, position, layers) in &added {
        let world_aabb = Aabb2D::new(position.0, aabb.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }

    // 3. Changes — position changed (skip entities just added this frame)
    for (entity, aabb_ref, position, layers) in &changed_pos {
        if aabb_ref.is_added() {
            continue;
        }
        quadtree.quadtree.remove(entity);
        let world_aabb = Aabb2D::new(position.0, aabb_ref.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }

    // 4. Changes — layers changed (skip entities just added or already updated by position change)
    for (entity, aabb_ref, position, layers) in &changed_layers {
        if aabb_ref.is_added() {
            continue;
        }
        // Only update if position did NOT also change (to avoid double-update).
        // If position also changed, the position loop already handled the remove+insert.
        if changed_pos.get(entity).is_ok() {
            continue;
        }
        quadtree.quadtree.remove(entity);
        let world_aabb = Aabb2D::new(position.0, aabb_ref.half_extents);
        quadtree.quadtree.insert(entity, world_aabb, *layers);
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CollisionQuadtree>();
        app.add_systems(FixedUpdate, maintain_quadtree);
        app
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 14: Entity with Aabb2D added is inserted ──────────

    #[test]
    fn added_entity_inserted_into_quadtree_at_world_space_position() {
        let mut app = test_app();
        // Initial tick to let MinimalPlugins initialize
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                Position2D(Vec2::new(100.0, 50.0)),
                CollisionLayers::new(0x02, 0x01),
            ))
            .id();

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(
            cq.quadtree.len(),
            1,
            "entity should be in quadtree after maintain_quadtree"
        );

        // The entity's world-space AABB should be centered at position
        let query_region = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(35.0, 12.0));
        let results = cq.quadtree.query_aabb(&query_region);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entity);
    }

    #[test]
    fn added_entity_at_origin_uses_zero_center() {
        let mut app = test_app();
        app.update();

        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            Position2D(Vec2::ZERO),
            CollisionLayers::new(0x01, 0x01),
        ));

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(cq.quadtree.len(), 1);
    }

    // ── Behavior 15: Position2D changed updates quadtree entry ─────

    #[test]
    fn position_changed_updates_quadtree_entry() {
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                Position2D(Vec2::new(100.0, 50.0)),
                CollisionLayers::new(0x01, 0x01),
            ))
            .id();

        // First tick: entity is added
        tick(&mut app);

        // Verify initial insertion
        {
            let cq = app.world().resource::<CollisionQuadtree>();
            assert_eq!(cq.quadtree.len(), 1);
        }

        // Move entity to new position
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Position2D>()
            .unwrap()
            .0 = Vec2::new(200.0, 50.0);

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        // Query at old position should NOT find entity
        let old_region = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(10.0, 10.0));
        let old_results = cq.quadtree.query_aabb(&old_region);
        assert!(
            old_results.is_empty(),
            "entity should not be at old position after move"
        );

        // Query at new position SHOULD find entity
        let new_region = Aabb2D::new(Vec2::new(200.0, 50.0), Vec2::new(10.0, 10.0));
        let new_results = cq.quadtree.query_aabb(&new_region);
        assert_eq!(new_results.len(), 1);
        assert_eq!(new_results[0], entity);
    }

    // ── Behavior 16: Aabb2D removed is removed from quadtree ───────

    #[test]
    fn aabb_removed_entity_removed_from_quadtree() {
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                Position2D(Vec2::new(50.0, 50.0)),
                CollisionLayers::new(0x01, 0x01),
            ))
            .id();

        // First tick: entity is added
        tick(&mut app);
        {
            let cq = app.world().resource::<CollisionQuadtree>();
            assert_eq!(cq.quadtree.len(), 1);
        }

        // Remove the Aabb2D component
        app.world_mut().entity_mut(entity).remove::<Aabb2D>();

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(
            cq.quadtree.len(),
            0,
            "entity should be removed from quadtree after Aabb2D removal"
        );
    }

    // ── Behavior 17: CollisionLayers changed updates quadtree ──────

    #[test]
    fn collision_layers_changed_updates_quadtree_entry() {
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                Position2D(Vec2::new(50.0, 50.0)),
                CollisionLayers::new(0x01, 0x02),
            ))
            .id();

        // First tick: entity is added
        tick(&mut app);

        // Change layers
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<CollisionLayers>()
            .unwrap() = CollisionLayers::new(0x02, 0x04);

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        // Old mask should not match
        let region = Aabb2D::new(Vec2::new(50.0, 50.0), Vec2::new(10.0, 10.0));
        let old_results = cq
            .quadtree
            .query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x01));
        assert!(
            old_results.is_empty(),
            "old membership=0x01 should no longer match after layer change"
        );

        // New mask should match
        let new_results = cq
            .quadtree
            .query_aabb_filtered(&region, CollisionLayers::new(0x00, 0x02));
        assert_eq!(
            new_results.len(),
            1,
            "new membership=0x02 should match after layer change"
        );
        assert_eq!(new_results[0], entity);
    }

    // ── Behavior 18: Added+Changed double-insert guard ─────────────

    #[test]
    fn newly_added_entity_not_double_inserted_on_first_frame() {
        let mut app = test_app();
        app.update();

        // Spawn two entities in the same frame
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            Position2D(Vec2::new(10.0, 10.0)),
            CollisionLayers::new(0x01, 0x01),
        ));
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            Position2D(Vec2::new(20.0, 20.0)),
            CollisionLayers::new(0x01, 0x01),
        ));

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(
            cq.quadtree.len(),
            2,
            "two entities should be inserted exactly once each (not 4 from double-insert)"
        );
    }

    // ── Behavior 19: Processing order — removals first, then additions ──

    #[test]
    fn removals_processed_before_additions() {
        let mut app = test_app();
        app.update();

        // Insert entity_a
        let entity_a = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
                Position2D(Vec2::new(10.0, 10.0)),
                CollisionLayers::new(0x01, 0x01),
            ))
            .id();

        tick(&mut app);
        {
            let cq = app.world().resource::<CollisionQuadtree>();
            assert_eq!(cq.quadtree.len(), 1);
        }

        // Same frame: remove entity_a's Aabb2D and add entity_b
        app.world_mut().entity_mut(entity_a).remove::<Aabb2D>();
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            Position2D(Vec2::new(30.0, 30.0)),
            CollisionLayers::new(0x02, 0x02),
        ));

        tick(&mut app);

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(
            cq.quadtree.len(),
            1,
            "entity_a removed + entity_b added = net 1 entity in quadtree"
        );
    }
}
