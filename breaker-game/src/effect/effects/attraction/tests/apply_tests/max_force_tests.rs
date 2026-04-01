//! Tests for `max_force` clamping behavior: force exceeds cap, at cap exactly,
//! below cap, `None` `max_force`, multiple entries with nearest entry's cap,
//! and `Some(0.0)` disabling steering.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{BaseSpeed, GlobalPosition2D, Velocity2D};

use super::super::{super::effect::*, *};
use crate::{
    effect::core::AttractionType,
    shared::{CELL_LAYER, WALL_LAYER},
};

/// Computes the expected velocity after one tick of attraction steering.
///
/// Same math as the production system: blend direction toward target,
/// normalize, then set magnitude to `base_speed`.
fn expected_velocity(
    starting_vel: Vec2,
    direction_to_target: Vec2,
    effective_force: f32,
    dt: f32,
    base_speed: f32,
) -> Vec2 {
    let steering = direction_to_target * effective_force * dt;
    let new_dir = (starting_vel + steering).normalize_or_zero();
    new_dir * base_speed
}

// ── Behavior 12: apply_attraction clamps force to max_force when Some (force exceeds cap) ──

#[test]
fn apply_attraction_clamps_force_to_max_force_when_exceeded() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Start moving upward so force differences produce different steering angles.
    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    let expected = expected_velocity(start_vel, Vec2::X, 200.0, dt, base_speed);
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "force should be clamped to max_force 200.0: expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}

#[test]
fn apply_attraction_at_cap_exactly_applies_unchanged() {
    let mut app = test_app();
    enter_playing(&mut app);

    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    let expected = expected_velocity(start_vel, Vec2::X, 200.0, dt, base_speed);
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "force at cap exactly should match: expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}

// ── Behavior 13: apply_attraction does not clamp when force is below max_force ──

#[test]
fn apply_attraction_does_not_clamp_when_force_below_max_force() {
    let mut app = test_app();
    enter_playing(&mut app);

    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    let expected = expected_velocity(start_vel, Vec2::X, 100.0, dt, base_speed);
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "force below cap should not be clamped: expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}

// ── Behavior 14: apply_attraction does not clamp when max_force is None ──

#[test]
fn apply_attraction_does_not_clamp_when_max_force_is_none() {
    let mut app = test_app();
    enter_playing(&mut app);

    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    let expected = expected_velocity(start_vel, Vec2::X, 1000.0, dt, base_speed);
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "None max_force should not clamp: expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}

// ── Behavior 15: apply_attraction with multiple entries uses winning entry's max_force ──

#[test]
fn apply_attraction_multiple_entries_uses_nearest_entry_max_force() {
    let mut app = test_app();
    enter_playing(&mut app);

    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    let expected = expected_velocity(start_vel, Vec2::X, 500.0, dt, base_speed);
    assert!(
        velocity.x > 0.0,
        "should steer toward nearest target (wall at 50,0), got velocity.x = {}",
        velocity.x
    );
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "nearest entry (wall) has max_force: None, so force 500.0 not clamped: expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}

// ── Behavior 16: apply_attraction clamps to max_force when max_force is Some(0.0) ──

#[test]
fn apply_attraction_max_force_zero_disables_steering() {
    let mut app = test_app();
    enter_playing(&mut app);

    let start_vel = Vec2::new(0.0, 10000.0);
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(start_vel),
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
    let base_speed = app.world().get::<BaseSpeed>(entity_a).unwrap().0;
    // max_force: Some(0.0) — zero steering, direction unchanged (still pointing up)
    // apply_velocity_formula sets magnitude to `base_speed` in the same direction
    let expected = Vec2::new(0.0, base_speed);
    assert!(
        (velocity.0 - expected).length() < 1.0,
        "max_force: Some(0.0) should not steer, expected {:?}, got {:?}",
        expected,
        velocity.0
    );
}
