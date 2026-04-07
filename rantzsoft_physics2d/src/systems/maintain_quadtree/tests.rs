use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D};

use super::system::*;
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

// -- Behavior 1: Entity inserted at GlobalPosition2D ---------
// Quadtree should use GlobalPosition2D (world-space), NOT Position2D (local).
// We set Position2D to a clearly wrong value (999.0, 999.0) to prove the
// system reads GlobalPosition2D instead.

#[test]
fn entity_inserted_at_global_position_not_local() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            GlobalPosition2D(Vec2::new(100.0, 50.0)),
            Position2D(Vec2::new(999.0, 999.0)),
            CollisionLayers::new(0x01, 0x01),
        ))
        .id();

    tick(&mut app);

    let cq = app.world().resource::<CollisionQuadtree>();
    assert_eq!(
        cq.quadtree.len(),
        1,
        "entity should be in quadtree after maintain_quadtree"
    );

    // Query around GlobalPosition2D (100.0, 50.0) -- should find entity
    let global_region = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(10.0, 10.0));
    let global_results = cq.quadtree.query_aabb(&global_region);
    assert_eq!(
        global_results.len(),
        1,
        "entity should be found at GlobalPosition2D (100.0, 50.0)"
    );
    assert_eq!(global_results[0], entity);

    // Query around Position2D (999.0, 999.0) -- should NOT find entity
    let local_region = Aabb2D::new(Vec2::new(999.0, 999.0), Vec2::new(10.0, 10.0));
    let local_results = cq.quadtree.query_aabb(&local_region);
    assert!(
        local_results.is_empty(),
        "entity should NOT be found at Position2D (999.0, 999.0) — \
         quadtree should use GlobalPosition2D, not Position2D"
    );
}

// -- Behavior 2: GlobalPosition2D change updates quadtree --------

#[test]
fn global_position_change_updates_quadtree_entry() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            GlobalPosition2D(Vec2::new(100.0, 50.0)),
            Position2D(Vec2::new(100.0, 50.0)),
            CollisionLayers::new(0x01, 0x01),
        ))
        .id();

    // First tick: entity is added
    tick(&mut app);
    {
        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(cq.quadtree.len(), 1);
    }

    // Change GlobalPosition2D to new location
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<GlobalPosition2D>()
        .unwrap()
        .0 = Vec2::new(200.0, 50.0);

    tick(&mut app);

    let cq = app.world().resource::<CollisionQuadtree>();

    // Query at old GlobalPosition2D -- should NOT find entity
    let old_region = Aabb2D::new(Vec2::new(100.0, 50.0), Vec2::new(10.0, 10.0));
    let old_results = cq.quadtree.query_aabb(&old_region);
    assert!(
        old_results.is_empty(),
        "entity should not be at old GlobalPosition2D after move"
    );

    // Query at new GlobalPosition2D -- should find entity
    let new_region = Aabb2D::new(Vec2::new(200.0, 50.0), Vec2::new(10.0, 10.0));
    let new_results = cq.quadtree.query_aabb(&new_region);
    assert_eq!(new_results.len(), 1);
    assert_eq!(new_results[0], entity);
}

// -- Behavior 14: Entity with Aabb2D added is inserted ----------

#[test]
fn added_entity_inserted_into_quadtree_at_world_space_position() {
    let mut app = test_app();
    // Initial tick to let MinimalPlugins initialize
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
            GlobalPosition2D(Vec2::new(100.0, 50.0)),
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

    // The entity's world-space AABB should be centered at GlobalPosition2D
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
        GlobalPosition2D(Vec2::ZERO),
        Position2D(Vec2::ZERO),
        CollisionLayers::new(0x01, 0x01),
    ));

    tick(&mut app);

    let cq = app.world().resource::<CollisionQuadtree>();
    assert_eq!(cq.quadtree.len(), 1);
}

// -- Behavior 15: GlobalPosition2D changed updates quadtree entry -

#[test]
fn position_changed_updates_quadtree_entry() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            GlobalPosition2D(Vec2::new(100.0, 50.0)),
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

    // Move entity to new position (update both for consistency,
    // but the system should read GlobalPosition2D)
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<GlobalPosition2D>()
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

// -- Behavior 16: Aabb2D removed is removed from quadtree -------

#[test]
fn aabb_removed_entity_removed_from_quadtree() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            GlobalPosition2D(Vec2::new(50.0, 50.0)),
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

// -- Behavior 17: CollisionLayers changed updates quadtree ------

#[test]
fn collision_layers_changed_updates_quadtree_entry() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            GlobalPosition2D(Vec2::new(50.0, 50.0)),
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

// -- Behavior 18: Added+Changed double-insert guard -------------

#[test]
fn newly_added_entity_not_double_inserted_on_first_frame() {
    let mut app = test_app();
    app.update();

    // Spawn two entities in the same frame
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        GlobalPosition2D(Vec2::new(10.0, 10.0)),
        Position2D(Vec2::new(10.0, 10.0)),
        CollisionLayers::new(0x01, 0x01),
    ));
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        GlobalPosition2D(Vec2::new(20.0, 20.0)),
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

// -- Behavior 19: Processing order -- removals first, then additions --

#[test]
fn removals_processed_before_additions() {
    let mut app = test_app();
    app.update();

    // Insert entity_a
    let entity_a = app
        .world_mut()
        .spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            GlobalPosition2D(Vec2::new(10.0, 10.0)),
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
        GlobalPosition2D(Vec2::new(30.0, 30.0)),
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
