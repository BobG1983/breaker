use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use super::helpers::*;
use crate::{
    cells::components::Cell,
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer, NodeScalingFactor, WALL_LAYER},
    walls::components::Wall,
};

// --- NodeScalingFactor collision tests ---

#[test]
fn scaled_bolt_effective_radius_changes_cell_collision_boundary() {
    let mut app = test_app();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = 81.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 50.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(NodeScalingFactor(0.5));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "scaled bolt (effective_radius=4) at y=81 should NOT reach cell (expanded bottom=84), \
         got vy={:.1} (if negative, full radius expansion was used instead of scaled)",
        vel.0.y
    );
}

#[test]
fn bolt_without_entity_scale_in_cell_collision_is_backward_compatible() {
    // Same as bolt_reflects_off_cell_bottom but explicitly no NodeScalingFactor.
    // Bolt should use full radius (8.0) and reflect normally.
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 5.0;
    // No NodeScalingFactor component
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt without NodeScalingFactor should reflect normally, got vy={:.1}",
        vel.0.y
    );
}

// --- Quadtree broad-phase collision tests ---
//
// These tests verify that the CCD system reads collision extents from
// `Aabb2D.half_extents` (populated by the quadtree broad phase) rather
// than from legacy dimension components.

#[test]
fn ccd_reads_cell_half_extents_from_aabb2d_not_cell_dimensions() {
    // Cell at (0, 100) with standard CellWidth(70)/CellHeight(24)
    // (half_extents 35.0, 12.0) but Aabb2D half_extents set to (5.0, 5.0).
    //
    // Bolt at (20, start_y) moving upward. x=20 is:
    //  - INSIDE the CellWidth-based expanded AABB (35 + 8 = 43)
    //  - OUTSIDE the Aabb2D-based expanded AABB (5 + 8 = 13)
    //
    // If the system reads from Aabb2D, the bolt misses (no reflection).
    // If the system reads from CellWidth/CellHeight, the bolt hits and reflects.
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let _cell = spawn_cell_with_custom_aabb(
        &mut app,
        0.0,
        cell_y,
        Vec2::new(5.0, 5.0), // tiny AABB
    );

    // Bolt at x=20, well within CellWidth range (half=35) but outside
    // Aabb2D range (half=5). Place below the cell's bottom.
    let expanded_bottom = cell_y - cc.height / 2.0 - bc.radius;
    let start_y = expanded_bottom - 2.0;
    spawn_bolt(&mut app, 20.0, start_y, 0.0, 400.0);

    // Run one tick to populate quadtree, then another for collision
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt at x=20 should miss the cell when CCD reads Aabb2D(5,5) \
         instead of CellWidth(70)/CellHeight(24) — got vy={:.1} \
         (negative means it reflected off the cell using legacy dimensions)",
        vel.0.y
    );
}

#[test]
fn ccd_reads_wall_half_extents_from_aabb2d() {
    // Wall at (200, 0) with tiny Aabb2D half_extents (5.0, 5.0).
    //
    // Bolt at (137, 50) moving right at (400, 0.1). y=50 is:
    //  - would be inside a legacy large-AABB expanded Y range
    //  - OUTSIDE the Aabb2D-based expanded Y range (-13 to 13)
    //
    // If the system reads from Aabb2D, the bolt misses (y=50 outside range).
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();

    // Spawn wall with tiny Aabb2D
    app.world_mut().spawn((
        Wall,
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        Position2D(Vec2::new(200.0, 0.0)),
        GlobalPosition2D(Vec2::new(200.0, 0.0)),
        Spatial2D,
        GameDrawLayer::Wall,
    ));

    // Bolt outside the expanded AABB on the left (x=137 < 142=200-50-8)
    // but at y=50 which is outside the Aabb2D expanded range.
    let start_x = 200.0 - 50.0 - bc.radius - 5.0; // 137.0
    spawn_bolt(&mut app, start_x, 50.0, 400.0, 0.1);

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.x > 0.0,
        "bolt at y=50 should miss the wall when CCD reads Aabb2D(5,5) \
         — got vx={:.1} \
         (negative means it reflected off the wall using incorrect dimensions)",
        vel.0.x
    );
}

#[test]
fn ccd_uses_aabb2d_larger_than_cell_dimensions_to_detect_hit() {
    // Inverse test: cell has small CellWidth(70)/CellHeight(24) but
    // large Aabb2D half_extents (100.0, 50.0).
    //
    // Bolt at (60, start_y) moving upward. x=60 is:
    //  - OUTSIDE the CellWidth-based expanded AABB (35 + 8 = 43)
    //  - INSIDE the Aabb2D-based expanded AABB (100 + 8 = 108)
    //
    // If the system reads from Aabb2D, the bolt hits and reflects.
    // If the system reads from CellWidth/CellHeight, the bolt misses.
    let mut app = test_app();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    let cell_entity = spawn_cell_with_custom_aabb(
        &mut app,
        0.0,
        cell_y,
        Vec2::new(100.0, 50.0), // large AABB
    );

    // Place bolt at x=60, outside CellWidth range but inside Aabb2D range
    // Aabb2D expanded bottom: 100 - 50 - 8 = 42
    let start_y = 42.0 - 2.0; // just below the Aabb2D expanded bottom
    spawn_bolt(&mut app, 60.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        1,
        "bolt at x=60 should hit the cell when CCD uses Aabb2D(100,50) — \
         got {} hits (0 means it used legacy CellWidth/CellHeight instead)",
        hits.0.len()
    );
    assert_eq!(
        hits.0[0], cell_entity,
        "the hit entity should be the cell with the large Aabb2D"
    );
}

#[test]
fn cell_with_aabb2d_but_no_cell_dimensions_is_collision_candidate() {
    // A cell entity with `Cell`, `Aabb2D`, `CollisionLayers`, and
    // `Position2D` but WITHOUT `CellWidth`/`CellHeight` components.
    //
    // The refactored system reads collision extents from `Aabb2D`, so
    // this cell IS a collision candidate even without the legacy
    // dimension components.
    //
    // The current system uses `Query<CollisionQueryCell>` which requires
    // `CellWidth`/`CellHeight` — so this cell is invisible to it.
    let mut app = test_app();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let bc = super::helpers::test_bolt_definition();

    // Spawn a cell with ONLY Aabb2D (no CellWidth/CellHeight)
    let cell_y = 100.0;
    let half_extents = Vec2::new(35.0, 12.0);
    let cell_entity = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(Vec2::new(0.0, cell_y)),
            GlobalPosition2D(Vec2::new(0.0, cell_y)),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id();

    // Bolt approaching from below
    let expanded_bottom = cell_y - half_extents.y - bc.radius;
    let start_y = expanded_bottom - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        1,
        "cell with Aabb2D but no CellWidth/CellHeight should still be a collision \
         candidate when the system reads from Aabb2D — got {} hits \
         (0 means the system still requires CellWidth/CellHeight)",
        hits.0.len()
    );
    assert_eq!(
        hits.0[0], cell_entity,
        "the hit should be the cell with only Aabb2D"
    );
}
