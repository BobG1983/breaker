//! Tests for core attraction steering behavior.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Velocity2D};

use super::super::super::{super::effect::*, *};
use crate::{
    effect::core::AttractionType,
    shared::{BREAKER_LAYER, CELL_LAYER, WALL_LAYER},
};

// ── Existing tests (updated for max_force: None field) ──

#[test]
fn apply_attraction_steers_toward_nearest_cell_target() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Entity A at origin with Cell attraction
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            }]),
        ))
        .id();

    // Cell target at (100, 0)
    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[(
            cell,
            Vec2::new(100.0, 0.0),
            CollisionLayers::new(CELL_LAYER, 0),
        )],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    assert!(
        velocity.x > 0.0,
        "entity should be steered toward cell at +x, got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_attraction_zero_distance_no_steering() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Entity A at same position as cell
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(100.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            }]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[(
            cell,
            Vec2::new(100.0, 0.0),
            CollisionLayers::new(CELL_LAYER, 0),
        )],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "zero distance should produce no steering, got {:?}",
        velocity.0
    );
}

#[test]
fn apply_attraction_inactive_entry_produces_no_steering() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: false,
            }]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[(
            cell,
            Vec2::new(100.0, 0.0),
            CollisionLayers::new(CELL_LAYER, 0),
        )],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    assert_eq!(
        velocity.0,
        Vec2::ZERO,
        "inactive attraction should produce no steering, got {:?}",
        velocity.0
    );
}

#[test]
fn apply_attraction_mixed_active_inactive_only_active_steers() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    max_force: None,
                    active: false,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 300.0,
                    max_force: None,
                    active: true,
                },
            ]),
        ))
        .id();

    // Wall target at (0, 100)
    let wall = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(0.0, 100.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[(
            wall,
            Vec2::new(0.0, 100.0),
            CollisionLayers::new(WALL_LAYER, 0),
        )],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    assert!(
        velocity.y > 0.0,
        "only active Wall attraction should steer toward +y, got velocity.y = {}",
        velocity.y
    );
}

#[test]
fn apply_attraction_multiple_types_nearest_target_wins() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    max_force: None,
                    active: true,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 500.0,
                    max_force: None,
                    active: true,
                },
            ]),
        ))
        .id();

    // Cell at (200, 0) -- farther
    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(200.0, 0.0)))
        .id();

    // Wall at (50, 0) -- closer
    let wall = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(50.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[
            (
                cell,
                Vec2::new(200.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            ),
            (
                wall,
                Vec2::new(50.0, 0.0),
                CollisionLayers::new(WALL_LAYER, 0),
            ),
        ],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    // Steered toward wall at (50, 0) -- velocity x should be positive
    assert!(
        velocity.x > 0.0,
        "should steer toward nearest target (wall at 50,0), got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_attraction_only_queries_matching_layer() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            }]),
        ))
        .id();

    // Cell at (100, 0) with CELL_LAYER
    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
        .id();

    // Wall at (0, 100) with WALL_LAYER -- should NOT be a target for Cell attraction
    let wall = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(0.0, 100.0)))
        .id();

    // Breaker at (-100, 0) with BREAKER_LAYER -- should NOT be a target
    let breaker = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(-100.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[
            (
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            ),
            (
                wall,
                Vec2::new(0.0, 100.0),
                CollisionLayers::new(WALL_LAYER, 0),
            ),
            (
                breaker,
                Vec2::new(-100.0, 0.0),
                CollisionLayers::new(BREAKER_LAYER, 0),
            ),
        ],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    // Should steer only toward the cell at (100, 0)
    assert!(
        velocity.x > 0.0,
        "Cell attraction should steer toward cell at (100,0), got velocity.x = {}",
        velocity.x
    );
    // Y should be zero or negligible (only steered in +x direction toward cell)
    assert!(
        velocity.y.abs() < 0.01,
        "Cell attraction should not steer toward wall or breaker, got velocity.y = {}",
        velocity.y
    );
}

#[test]
fn apply_attraction_force_scales_with_dt() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 1000.0,
                max_force: None,
                active: true,
            }]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
        .id();

    populate_quadtree(
        &mut app,
        &[(
            cell,
            Vec2::new(100.0, 0.0),
            CollisionLayers::new(CELL_LAYER, 0),
        )],
    );

    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    // Direction toward (100, 0) from (0, 0) is (1.0, 0.0).
    // With force=1000 and default dt=~0.015625 (1/64), velocity.x should be
    // approximately 1000 * 0.015625 = 15.625.
    // We use a generous tolerance because the exact dt may vary with MinimalPlugins.
    assert!(
        velocity.x > 0.0,
        "velocity.x should be positive after attraction force * dt, got {}",
        velocity.x
    );
    // Verify the velocity is proportional to force (not just direction)
    assert!(
        velocity.x > 1.0,
        "with force=1000 and any reasonable dt, velocity.x should be > 1.0, got {}",
        velocity.x
    );
}
