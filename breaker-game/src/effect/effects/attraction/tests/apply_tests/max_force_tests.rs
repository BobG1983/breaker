//! Tests for `max_force` clamping behavior: force exceeds cap, at cap exactly,
//! below cap, `None` `max_force`, multiple entries with nearest entry's cap,
//! and `Some(0.0)` disabling steering.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Velocity2D};

use super::super::{super::effect::*, *};
use crate::{
    effect::core::AttractionType,
    shared::{CELL_LAYER, WALL_LAYER},
};

// ── Behavior 12: apply_attraction clamps force to max_force when Some (force exceeds cap) ──

#[test]
fn apply_attraction_clamps_force_to_max_force_when_exceeded() {
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
                max_force: Some(200.0),
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
    // Effective force should be min(1000.0, 200.0) = 200.0
    // Velocity x should be approximately 200.0 * dt
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_vx = 200.0 * dt;
    assert!(
        (velocity.x - expected_vx).abs() < 0.1,
        "force should be clamped to max_force 200.0: expected velocity.x ~ {expected_vx}, got {}",
        velocity.x
    );
}

#[test]
fn apply_attraction_at_cap_exactly_applies_unchanged() {
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
                force: 200.0,
                max_force: Some(200.0),
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
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_vx = 200.0 * dt;
    assert!(
        (velocity.x - expected_vx).abs() < 0.1,
        "force at cap exactly should apply unchanged: expected velocity.x ~ {expected_vx}, got {}",
        velocity.x
    );
}

// ── Behavior 13: apply_attraction does not clamp when force is below max_force ──

#[test]
fn apply_attraction_does_not_clamp_when_force_below_max_force() {
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
                force: 100.0,
                max_force: Some(200.0),
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
    // Effective force should be min(100.0, 200.0) = 100.0 (cap not triggered)
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_vx = 100.0 * dt;
    assert!(
        (velocity.x - expected_vx).abs() < 0.1,
        "force below cap should not be clamped: expected velocity.x ~ {expected_vx}, got {}",
        velocity.x
    );
}

// ── Behavior 14: apply_attraction does not clamp when max_force is None ──

#[test]
fn apply_attraction_does_not_clamp_when_max_force_is_none() {
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
    // No clamping -- force 1000.0 applied directly
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_vx = 1000.0 * dt;
    assert!(
        (velocity.x - expected_vx).abs() < 0.1,
        "None max_force should not clamp: expected velocity.x ~ {expected_vx}, got {}",
        velocity.x
    );
}

// ── Behavior 15: apply_attraction with multiple entries uses winning entry's max_force ──

#[test]
fn apply_attraction_multiple_entries_uses_nearest_entry_max_force() {
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
                    force: 800.0,
                    max_force: Some(100.0),
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

    // Wall at (50, 0) -- closer (wins)
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
    // Nearest is wall at (50, 0) with force=500.0, max_force=None (no clamp)
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected_vx = 500.0 * dt;
    assert!(
        velocity.x > 0.0,
        "should steer toward nearest target (wall at 50,0), got velocity.x = {}",
        velocity.x
    );
    assert!(
        (velocity.x - expected_vx).abs() < 0.1,
        "nearest entry (wall) has max_force: None, so force 500.0 not clamped: expected ~ {expected_vx}, got {}",
        velocity.x
    );
}

// ── Behavior 16: apply_attraction clamps to max_force when max_force is Some(0.0) ──

#[test]
fn apply_attraction_max_force_zero_disables_steering() {
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
                max_force: Some(0.0),
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
    // max_force: Some(0.0) should clamp force to 0.0, resulting in no velocity change
    assert!(
        velocity.x.abs() < f32::EPSILON,
        "max_force: Some(0.0) should disable steering, got velocity.x = {}",
        velocity.x
    );
    assert!(
        velocity.y.abs() < f32::EPSILON,
        "max_force: Some(0.0) should disable steering, got velocity.y = {}",
        velocity.y
    );
}
