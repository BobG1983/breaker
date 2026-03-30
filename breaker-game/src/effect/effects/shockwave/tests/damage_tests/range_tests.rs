//! Tests for shockwave damage range behavior: within/outside radius, dedup of
//! already-damaged cells, multiple cells, layer filtering, multiple shockwaves,
//! and zero radius.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::super::*;
use crate::{bolt::BASE_BOLT_DAMAGE, shared::WALL_LAYER};

// -- Behavior 1: Shockwave damages a cell within its current radius ──

#[test]
fn shockwave_damages_cell_within_radius() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, HashSet::new());

    // Tick to populate quadtree, then update to run damage system
    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
    assert!(
        (collector.0[0].damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
        "expected damage {}, got {}",
        BASE_BOLT_DAMAGE,
        collector.0[0].damage
    );
    assert!(
        collector.0[0].source_chip.is_none(),
        "source_chip should be None for shockwave damage"
    );

    // Cell should be in the shockwave's damaged set
    let damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert!(
        damaged.0.contains(&cell),
        "cell entity should be in ShockwaveDamaged set"
    );
}

// -- Behavior 2: Shockwave does not damage a cell outside its radius ──

#[test]
fn shockwave_does_not_damage_cell_outside_radius() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 200.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no DamageCell messages should be sent for cell outside radius"
    );

    let damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert!(damaged.0.is_empty(), "ShockwaveDamaged should remain empty");
}

// -- Behavior 3: Shockwave does not damage the same cell twice ──

#[test]
fn shockwave_does_not_damage_already_damaged_cell() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);
    let mut already_damaged = HashSet::new();
    already_damaged.insert(cell);
    let _shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, already_damaged);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "already-damaged cell should not receive DamageCell again"
    );
}

#[test]
fn shockwave_damages_new_cell_but_not_already_damaged_cell() {
    let mut app = damage_test_app();

    let cell_already = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_new = spawn_test_cell(&mut app, 25.0, 0.0);

    let mut already_damaged = HashSet::new();
    already_damaged.insert(cell_already);
    let _shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, already_damaged);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "only the new cell should be damaged, got {} messages",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].cell, cell_new,
        "only the new cell should receive damage"
    );
}

// -- Behavior 4: Shockwave damages multiple cells in range ──

#[test]
fn shockwave_damages_multiple_cells_in_range() {
    let mut app = damage_test_app();

    let cell1 = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell2 = spawn_test_cell(&mut app, 0.0, 15.0);
    let cell3 = spawn_test_cell(&mut app, -20.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());

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
            (msg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "each cell damage should be BASE_BOLT_DAMAGE (10.0)"
        );
    }

    let sw_damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert_eq!(
        sw_damaged.0.len(),
        3,
        "all three cells should be in ShockwaveDamaged set"
    );
}

#[test]
fn shockwave_only_damages_cell_layer_not_wall_layer() {
    let mut app = damage_test_app();

    // Spawn two cells on CELL_LAYER
    spawn_test_cell(&mut app, 10.0, 0.0);
    spawn_test_cell(&mut app, 0.0, 15.0);

    // Spawn one entity on WALL_LAYER (not CELL_LAYER)
    let wall_pos = Vec2::new(-20.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "only CELL_LAYER entities should be damaged, got {}",
        collector.0.len()
    );
}

// -- Behavior 5: Multiple shockwaves damage independently ──

#[test]
fn multiple_shockwaves_damage_independently() {
    let mut app = damage_test_app();

    let cell_near_sw1 = spawn_test_cell(&mut app, 15.0, 0.0);
    let cell_near_sw2 = spawn_test_cell(&mut app, 90.0, 0.0);

    let sw1 = spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());
    let sw2 = spawn_shockwave(&mut app, 100.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages (one per shockwave), got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(
        damaged_cells.contains(&cell_near_sw1),
        "cell near sw1 should be damaged"
    );
    assert!(
        damaged_cells.contains(&cell_near_sw2),
        "cell near sw2 should be damaged"
    );

    let sw1_damaged = app.world().get::<ShockwaveDamaged>(sw1).unwrap();
    assert!(
        sw1_damaged.0.contains(&cell_near_sw1),
        "sw1 should track cell_near_sw1"
    );
    assert!(
        !sw1_damaged.0.contains(&cell_near_sw2),
        "sw1 should NOT track cell_near_sw2"
    );

    let sw2_damaged = app.world().get::<ShockwaveDamaged>(sw2).unwrap();
    assert!(
        sw2_damaged.0.contains(&cell_near_sw2),
        "sw2 should track cell_near_sw2"
    );
    assert!(
        !sw2_damaged.0.contains(&cell_near_sw1),
        "sw2 should NOT track cell_near_sw1"
    );
}

#[test]
fn cell_midway_between_two_shockwaves_not_reached_by_either() {
    let mut app = damage_test_app();

    // Cell at midpoint (50, 0) -- neither shockwave with radius 25 reaches it
    spawn_test_cell(&mut app, 50.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());
    spawn_shockwave(&mut app, 100.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell at midpoint should not be reached by either shockwave"
    );
}

// -- Behavior 6: Shockwave with zero radius damages no cells ──

#[test]
fn shockwave_with_zero_radius_damages_no_cells() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 1.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 0.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "shockwave with radius 0.0 should damage no cells"
    );
}

#[test]
fn shockwave_with_zero_radius_at_same_position_as_cell_no_damage() {
    let mut app = damage_test_app();

    // Cell at same position as shockwave -- radius 0 means no matches
    spawn_test_cell(&mut app, 0.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 0.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "shockwave with radius 0.0 at cell position should still damage no cells"
    );
}
