//! Group G — cardinal slot occupancy (Behaviors 39–43).

use bevy::prelude::*;

use super::{
    super::helpers::{
        all_cells, approx_eq_vec2, cell_count, run_fixed_update, spawn_cell_at,
        spawn_cell_at_with_max, spawn_cell_dead_at, spawn_cell_invulnerable_at,
    },
    helpers::split_test_app,
};
use crate::prelude::*;

// ── Behavior 39 — 0 empty cardinal slots → no split, HP retained ────────────

#[test]
fn zero_empty_cardinal_slots_no_split_keeps_excess_hp() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // All 4 cardinal slots occupied by live cells.
    let _ = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(-70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(0.0, 24.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(0.0, -24.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "parent retains excess HP when no slots are empty; got {}",
        hp.current
    );
    // No new cells spawned — 5 total (1 parent + 4 occupiers).
    assert_eq!(cell_count(&mut app), 5);
}

#[test]
fn zero_empty_slots_still_no_split_next_tick() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    let _ = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(-70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(0.0, 24.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(0.0, -24.0), 5.0, 5.0);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
    assert_eq!(cell_count(&mut app), 5);
}

// ── Behavior 40 — 1 empty slot → 1 new cell spawned ─────────────────────────

#[test]
fn one_empty_slot_spawns_one_cell_and_resets_parent() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // 3 occupiers at right, left, up. DOWN slot (0, -24) is empty.
    let _ = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(-70.0, 0.0), 5.0, 5.0);
    let _ = spawn_cell_at(&mut app, Vec2::new(0.0, 24.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "parent resets when at least 1 slot was available; got {}",
        hp.current
    );
    // 5 cells total: 1 parent + 3 occupiers + 1 new.
    assert_eq!(cell_count(&mut app), 5);

    // Find the new cell (the one at (0, -24)).
    let cells = all_cells(&mut app);
    let new_cell = cells
        .iter()
        .find(|(_, p, _)| approx_eq_vec2(*p, Vec2::new(0.0, -24.0), 1e-4));
    assert!(
        new_cell.is_some(),
        "new cell must be at the empty down slot (0, -24)"
    );
    let (_, _, hp_new) = new_cell.unwrap();
    assert!((hp_new.current - 10.0).abs() < f32::EPSILON);
    assert!((hp_new.starting - 10.0).abs() < f32::EPSILON);
}

// ── Behavior 41 — 3 empty slots → still only 2 new cells (cap) ──────────────

#[test]
fn three_empty_slots_spawns_exactly_two_cells_cap() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // One occupier at right. Left, up, down all empty.
    let _ = spawn_cell_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    // 4 cells total: 1 parent + 1 occupier + 2 new.
    assert_eq!(cell_count(&mut app), 4);

    // The 2 new cells should be at the first 2 empty slots in iteration order:
    // LEFT (-70,0) and UP (0,24).
    let cells = all_cells(&mut app);
    let positions: Vec<Vec2> = cells
        .iter()
        .filter(|(e, p, _)| *e != parent && !approx_eq_vec2(*p, Vec2::new(70.0, 0.0), 1e-4))
        .map(|(_, p, _)| *p)
        .collect();
    assert_eq!(positions.len(), 2);
    let mut found_left = false;
    let mut found_up = false;
    for pos in &positions {
        if approx_eq_vec2(*pos, Vec2::new(-70.0, 0.0), 1e-4) {
            found_left = true;
        } else if approx_eq_vec2(*pos, Vec2::new(0.0, 24.0), 1e-4) {
            found_up = true;
        }
    }
    assert!(
        found_left && found_up,
        "new cells must be at left (-70,0) and up (0,24); got {positions:?}"
    );
}

// ── Behavior 42 — occupancy uses ADJACENCY_RADIUS_SQ proximity ──────────────

#[test]
fn occupier_near_cardinal_slot_counts_as_occupied() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // Occupier at (69.0, 0.0) — distance² from (70, 0) is 1.0 ≤ 4900.
    let _ = spawn_cell_at(&mut app, Vec2::new(69.0, 0.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    // 4 cells: parent + occupier + 2 new (at left and up; right is "occupied").
    assert_eq!(cell_count(&mut app), 4);

    let cells = all_cells(&mut app);
    let positions: Vec<Vec2> = cells
        .iter()
        .filter(|(e, p, _)| *e != parent && !approx_eq_vec2(*p, Vec2::new(69.0, 0.0), 1e-4))
        .map(|(_, p, _)| *p)
        .collect();
    assert_eq!(positions.len(), 2);
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(-70.0, 0.0), 1e-4))
    );
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(0.0, 24.0), 1e-4))
    );
}

#[test]
fn far_occupier_does_not_block_slot() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // Occupier at (700, 0) — way outside ADJACENCY_RADIUS_SQ of (70, 0).
    let _ = spawn_cell_at(&mut app, Vec2::new(700.0, 0.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    // 4 cells: parent + far occupier + 2 new (right AND left, because the far cell doesn't occupy right).
    assert_eq!(cell_count(&mut app), 4);

    let cells = all_cells(&mut app);
    let positions: Vec<Vec2> = cells
        .iter()
        .filter(|(e, p, _)| *e != parent && !approx_eq_vec2(*p, Vec2::new(700.0, 0.0), 1e-4))
        .map(|(_, p, _)| *p)
        .collect();
    assert_eq!(positions.len(), 2);
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(70.0, 0.0), 1e-4))
    );
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(-70.0, 0.0), 1e-4))
    );
}

// ── Behavior 43 — Dead cells in cardinal slots count as EMPTY ───────────────

#[test]
fn dead_cell_in_slot_counts_as_empty() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // Dead cell at (70, 0). Other 3 slots empty.
    let _ = spawn_cell_dead_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    // Dead cell still exists (split_check doesn't despawn). 4 cells total.
    assert_eq!(cell_count(&mut app), 4);

    // The 2 new cells should spawn at (70, 0) (dead slot treated as empty, first
    // in iteration order) and (-70, 0) (left, next).
    let cells = all_cells(&mut app);
    let live_new_positions: Vec<Vec2> = cells
        .iter()
        .filter(|(e, ..)| *e != parent && app.world().get::<crate::prelude::Dead>(*e).is_none())
        .map(|(_, p, _)| *p)
        .collect();
    assert_eq!(live_new_positions.len(), 2);
    assert!(
        live_new_positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(70.0, 0.0), 1e-4))
    );
    assert!(
        live_new_positions
            .iter()
            .any(|p| approx_eq_vec2(*p, Vec2::new(-70.0, 0.0), 1e-4))
    );
}

#[test]
fn invulnerable_cell_in_slot_counts_as_occupied() {
    let mut app = split_test_app();
    let _parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    // Invulnerable cell at (70, 0).
    let _ = spawn_cell_invulnerable_at(&mut app, Vec2::new(70.0, 0.0), 5.0, 5.0);

    run_fixed_update(&mut app);

    // 4 cells: parent + invulnerable occupier + 2 new (at left and up).
    assert_eq!(cell_count(&mut app), 4);
    let cells = all_cells(&mut app);
    // The right slot should NOT have a new non-invulnerable cell.
    let invuln_count_at_right = cells
        .iter()
        .filter(|(_, p, _)| approx_eq_vec2(*p, Vec2::new(70.0, 0.0), 1e-4))
        .count();
    assert_eq!(
        invuln_count_at_right, 1,
        "only the invulnerable occupier at right"
    );
}
