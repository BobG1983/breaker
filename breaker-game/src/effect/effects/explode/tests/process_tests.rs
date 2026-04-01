//! Tests for `process_explode_requests` system: damage in range, multiple cells,
//! outside range, despawn after processing, multiple requests, and layer filtering.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::resources::DEFAULT_BOLT_BASE_DAMAGE,
    cells::components::Cell,
    shared::{BOLT_LAYER, CELL_LAYER, WALL_LAYER},
};

// -- Behavior 21: process_explode_requests damages cells within range ──

#[test]
fn process_explode_requests_damages_cell_in_range() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);
    let request = spawn_explode_request(&mut app, 0.0, 0.0, 50.0, 2.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {}, got {}",
        expected_damage,
        collector.0[0].damage
    );
    assert!(
        collector.0[0].source_chip.is_none(),
        "source_chip should be None"
    );

    // Request entity should be despawned
    assert!(
        app.world().get_entity(request).is_err(),
        "ExplodeRequest entity should be despawned after processing"
    );
}

// -- Behavior 22: damages multiple cells in range ──

#[test]
fn process_explode_requests_damages_multiple_cells_in_range() {
    let mut app = damage_test_app();

    let cell1 = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell2 = spawn_test_cell(&mut app, 0.0, 20.0);
    let cell3 = spawn_test_cell(&mut app, -15.0, 0.0);
    let request = spawn_explode_request(&mut app, 0.0, 0.0, 30.0, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell1), "cell1 should be damaged");
    assert!(damaged_cells.contains(&cell2), "cell2 should be damaged");
    assert!(damaged_cells.contains(&cell3), "cell3 should be damaged");

    for msg in &collector.0 {
        assert!(
            (msg.damage - DEFAULT_BOLT_BASE_DAMAGE).abs() < f32::EPSILON,
            "damage should be DEFAULT_BOLT_BASE_DAMAGE * 1.0 = 10.0"
        );
    }

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );
}

#[test]
fn process_explode_requests_does_not_damage_cell_outside_range() {
    let mut app = damage_test_app();

    // Cell at (10, 0) -- in range of 30
    spawn_test_cell(&mut app, 10.0, 0.0);
    // Cell at (100, 0) -- outside range of 30
    spawn_test_cell(&mut app, 100.0, 0.0);

    spawn_explode_request(&mut app, 0.0, 0.0, 30.0, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "only cell in range should be damaged, got {}",
        collector.0.len()
    );
}

// -- Behavior 23: skips cells outside range ──

#[test]
fn process_explode_requests_sends_no_damage_for_distant_cell() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 200.0, 0.0);
    let request = spawn_explode_request(&mut app, 0.0, 0.0, 50.0, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no DamageCell messages for cells outside range"
    );

    // Request should still be despawned even with no hits
    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned even with no cells hit"
    );
}

// -- Behavior 24: despawns request entity after processing ──

#[test]
fn process_explode_requests_despawns_request_with_no_cells_in_range() {
    let mut app = damage_test_app();

    let request = spawn_explode_request(&mut app, 0.0, 0.0, 50.0, 1.0);

    tick(&mut app);

    assert!(
        app.world().get_entity(request).is_err(),
        "ExplodeRequest should be despawned even with no cells in range"
    );

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(collector.0.is_empty(), "no damage messages expected");
}

#[test]
fn process_explode_requests_handles_multiple_requests_in_same_frame() {
    let mut app = damage_test_app();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);
    let req1 = spawn_explode_request(&mut app, 0.0, 0.0, 30.0, 1.0);
    let req2 = spawn_explode_request(&mut app, 0.0, 0.0, 30.0, 2.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "each request should produce one DamageCell, got {}",
        collector.0.len()
    );

    // Both requests should be despawned
    assert!(
        app.world().get_entity(req1).is_err(),
        "first request should be despawned"
    );
    assert!(
        app.world().get_entity(req2).is_err(),
        "second request should be despawned"
    );

    // Verify different damage amounts — compare via sorted rounded values.
    let mut damages: Vec<f32> = collector
        .0
        .iter()
        .map(|m| (m.damage * 10.0).round())
        .collect();
    damages.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    damages.dedup();
    assert!(
        damages.iter().any(|&d| (d - 100.0).abs() < f32::EPSILON),
        "should have damage 10.0 from mult 1.0"
    );
    assert!(
        damages.iter().any(|&d| (d - 200.0).abs() < f32::EPSILON),
        "should have damage 20.0 from mult 2.0"
    );
}

// -- Behavior 25: only queries CELL_LAYER ──

#[test]
fn process_explode_requests_only_damages_cell_layer() {
    let mut app = damage_test_app();

    // Spawn bolt-layer entity (not a cell)
    let bolt_pos = Vec2::new(10.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    // Spawn wall-layer entity (not a cell)
    let wall_pos = Vec2::new(5.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    spawn_explode_request(&mut app, 0.0, 0.0, 50.0, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "non-CELL_LAYER entities should not be damaged"
    );
}

#[test]
fn process_explode_requests_damages_entity_with_cell_layer_in_combined_mask() {
    let mut app = damage_test_app();

    // Entity with CELL_LAYER | BOLT_LAYER -- should be damaged since it IS on CELL_LAYER
    let pos = Vec2::new(10.0, 0.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    spawn_explode_request(&mut app, 0.0, 0.0, 50.0, 1.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "entity with CELL_LAYER in combined mask should be damaged"
    );
    assert_eq!(collector.0[0].cell, cell);
}
