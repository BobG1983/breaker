//! Tests for edge cases: no targets, no component, empty list.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Velocity2D};

use super::super::super::{super::effect::*, helpers::*};
use crate::{effect::core::AttractionType, shared::CELL_LAYER};

#[test]
fn apply_attraction_no_targets_velocity_unchanged() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
            spatial_params(),
            ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                max_force: None,
                active: true,
            }]),
        ))
        .id();

    // Empty quadtree -- no targets
    app.update();

    let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
    assert!(
        (velocity.x - 100.0).abs() < f32::EPSILON,
        "velocity.x should be unchanged (100.0), got {}",
        velocity.x
    );
    assert!(
        (velocity.y - 200.0).abs() < f32::EPSILON,
        "velocity.y should be unchanged (200.0), got {}",
        velocity.y
    );
}

#[test]
fn apply_attraction_entity_without_attractions_unaffected() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Entity without ActiveAttractions
    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
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
    assert!(
        (velocity.x - 100.0).abs() < f32::EPSILON,
        "entity without ActiveAttractions should not be steered, got velocity.x = {}",
        velocity.x
    );
}

#[test]
fn apply_attraction_empty_attractions_no_steering() {
    let mut app = test_app();
    enter_playing(&mut app);

    let entity_a = app
        .world_mut()
        .spawn((
            GlobalPosition2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(100.0, 200.0)),
            spatial_params(),
            ActiveAttractions(vec![]),
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
    assert!(
        (velocity.x - 100.0).abs() < f32::EPSILON,
        "empty ActiveAttractions should produce no steering, got velocity.x = {}",
        velocity.x
    );
}
