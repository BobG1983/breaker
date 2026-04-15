//! Build tests — verify that dimension transitions store correct values for spawn.
//! Behaviors 4, 6, 8: `.position()`, `.dimensions()`, `.hp()` store values in spawned entity.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::{
    cells::{
        components::{Cell, CellHeight, CellWidth},
        test_utils::spawn_cell_in_world,
    },
    shared::death_pipeline::hp::Hp,
};

// ── Behavior 4: .position(pos) stores position for spawn ────────────────────

#[test]
fn position_stores_value_in_spawned_entity() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::new(100.0, 250.0))
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON && (pos.0.y - 250.0).abs() < f32::EPSILON,
        "Position2D should be (100.0, 250.0), got {:?}",
        pos.0
    );
}

// Behavior 4 edge case: zero position
#[test]
fn position_zero_stores_in_spawned_entity() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert_eq!(pos.0, Vec2::ZERO, "Position2D should be Vec2::ZERO");
}

// ── Behavior 6: .dimensions(w, h) stores width and height ───────────────────

#[test]
fn dimensions_stores_width_and_height_in_spawned_entity() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let width = world
        .get::<CellWidth>(entity)
        .expect("entity should have CellWidth");
    assert!(
        (width.value - 70.0).abs() < f32::EPSILON,
        "CellWidth should be 70.0, got {}",
        width.value
    );

    let height = world
        .get::<CellHeight>(entity)
        .expect("entity should have CellHeight");
    assert!(
        (height.value - 24.0).abs() < f32::EPSILON,
        "CellHeight should be 24.0, got {}",
        height.value
    );

    let scale = world
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        (scale.x - 70.0).abs() < f32::EPSILON && (scale.y - 24.0).abs() < f32::EPSILON,
        "Scale2D should be (70.0, 24.0), got ({}, {})",
        scale.x,
        scale.y
    );

    let aabb = world
        .get::<Aabb2D>(entity)
        .expect("entity should have Aabb2D");
    assert!(
        (aabb.half_extents.x - 35.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 12.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (35.0, 12.0), got {:?}",
        aabb.half_extents
    );
}

// Behavior 6 edge case: 1x1 dimensions
#[test]
fn dimensions_one_by_one_stores_correctly() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(1.0, 1.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let width = world
        .get::<CellWidth>(entity)
        .expect("entity should have CellWidth");
    assert!(
        (width.value - 1.0).abs() < f32::EPSILON,
        "CellWidth should be 1.0"
    );

    let height = world
        .get::<CellHeight>(entity)
        .expect("entity should have CellHeight");
    assert!(
        (height.value - 1.0).abs() < f32::EPSILON,
        "CellHeight should be 1.0"
    );

    let scale = world
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON && (scale.y - 1.0).abs() < f32::EPSILON,
        "Scale2D should be (1.0, 1.0)"
    );

    let aabb = world
        .get::<Aabb2D>(entity)
        .expect("entity should have Aabb2D");
    assert!(
        (aabb.half_extents.x - 0.5).abs() < f32::EPSILON
            && (aabb.half_extents.y - 0.5).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (0.5, 0.5)"
    );
}

// ── Behavior 8: .hp(value) stores health for spawn ──────────────────────────

#[test]
fn hp_stores_health_in_spawned_entity() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let health = world.get::<Hp>(entity).expect("entity should have Hp");
    assert!(
        (health.current - 20.0).abs() < f32::EPSILON
            && (health.starting - 20.0).abs() < f32::EPSILON,
        "Hp should be {{ current: 20.0, starting: 20.0 }}, got {{ current: {}, starting: {} }}",
        health.current,
        health.starting
    );
}

// Behavior 8 edge case: hp 1.0
#[test]
fn hp_one_stores_health_correctly() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(1.0)
            .headless()
            .spawn(commands)
    });

    let health = world.get::<Hp>(entity).expect("entity should have Hp");
    assert!(
        (health.current - 1.0).abs() < f32::EPSILON && (health.starting - 1.0).abs() < f32::EPSILON,
        "Hp should be {{ current: 1.0, starting: 1.0 }}"
    );
}
