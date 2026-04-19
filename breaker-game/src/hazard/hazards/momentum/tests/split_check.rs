//! Groups F + G — `momentum_split_check` (Behaviors 32–50, 71).
//!
//! Pins threshold detection (`hp.current >= hp.starting * 2.0`), cardinal
//! slot occupancy with `ADJACENCY_RADIUS_SQ` proximity, 2-cell spawn cap,
//! full collision suite on spawned cells, independent splits per tick,
//! `hp.max` preservation on split, and run-if gating.

use bevy::prelude::*;

use super::{
    super::system::{
        MOMENTUM_CELL_HEIGHT, MOMENTUM_CELL_WIDTH, MomentumConfig, momentum_split_check, register,
    },
    helpers::{
        add_momentum_stacks, all_cells, approx_eq_vec2, canonical_momentum_config, cell_count,
        install_momentum_config, run_fixed_update, spawn_cell_at, spawn_cell_at_with_max,
        spawn_cell_dead_at, spawn_cell_invulnerable_at, test_app_playing,
    },
};
use crate::{
    cells::components::{CellHeight, CellWidth},
    prelude::*,
    shared::{
        collision_layers::{BOLT_LAYER, CELL_LAYER},
        death_pipeline::{Hp, KilledBy},
    },
};

// ────────────────────────────────────────────────────────────────────────────
// Helpers specific to split-check tests
// ────────────────────────────────────────────────────────────────────────────

/// Builds a split-check-only app: state hierarchy in Playing, `ActiveHazards`,
/// `MomentumConfig`, and `momentum_split_check` wired in `FixedUpdate`.
fn split_test_app() -> App {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_split_check);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);
    app
}

// ════════════════════════════════════════════════════════════════════════════
// Group F — threshold detection
// ════════════════════════════════════════════════════════════════════════════

// ── Behavior 32 — cell at 2 × starting triggers split (inclusive boundary) ──

#[test]
fn cell_at_exact_threshold_triggers_split() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    // Parent resets to starting.
    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "parent hp.current must reset to 10.0 after split; got {}",
        hp.current
    );

    // Exactly 3 cells exist total (1 parent + 2 spawned).
    assert_eq!(cell_count(&mut app), 3);

    // Two new cells at (+70, 0) and (-70, 0) — right then left (iteration order).
    let cells = all_cells(&mut app);
    let positions: Vec<Vec2> = cells
        .iter()
        .filter(|(e, ..)| *e != parent)
        .map(|(_, p, _)| *p)
        .collect();
    assert_eq!(positions.len(), 2);
    for pos in &positions {
        assert!(
            approx_eq_vec2(*pos, Vec2::new(70.0, 0.0), 1e-4)
                || approx_eq_vec2(*pos, Vec2::new(-70.0, 0.0), 1e-4),
            "new cell at unexpected position: {pos:?}"
        );
    }

    // New cells have starting 10.0 and current 10.0.
    for (e, _, hp) in &cells {
        if *e != parent {
            assert!((hp.current - 10.0).abs() < f32::EPSILON);
            assert!((hp.starting - 10.0).abs() < f32::EPSILON);
        }
    }
}

#[test]
fn cell_just_below_threshold_does_not_trigger() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 19.999, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 19.999).abs() < 1e-4,
        "parent hp.current must be unchanged (19.999); got {}",
        hp.current
    );
    assert_eq!(cell_count(&mut app), 1, "no new cells should spawn");
}

// ── Behavior 33 — below threshold: no split ─────────────────────────────────

#[test]
fn below_threshold_no_split() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 19.999, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 19.999).abs() < 1e-4);
    assert_eq!(cell_count(&mut app), 1);
}

// ── Behavior 34 — well above threshold still resets to starting ─────────────

#[test]
fn above_threshold_three_x_resets_to_starting() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 30.0, 10.0, Some(50.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "parent current must reset to starting (10.0), not retain excess; got {}",
        hp.current
    );
    // 2 new cells spawned.
    assert_eq!(cell_count(&mut app), 3);
}

#[test]
fn above_threshold_ten_x_resets_to_starting() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 100.0, 10.0, Some(200.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert_eq!(cell_count(&mut app), 3);
}

// ── Behavior 35 — below threshold stays below across ticks ──────────────────

#[test]
fn below_threshold_never_splits_over_three_ticks() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 15.0, 10.0, Some(20.0));

    for _ in 0..3 {
        run_fixed_update(&mut app);
    }

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 15.0).abs() < f32::EPSILON);
    assert_eq!(cell_count(&mut app), 1);
}

// ── Behavior 36 — Dead cell at threshold does not split ─────────────────────

#[test]
fn dead_cell_at_threshold_does_not_split() {
    let mut app = split_test_app();
    let parent = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp {
                current:  20.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
            crate::shared::death_pipeline::Dead,
        ))
        .id();

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Dead cell's hp.current must be unchanged; got {}",
        hp.current
    );
    // Only the parent remains — no new cells.
    assert_eq!(cell_count(&mut app), 1);
}

// ── Behavior 37 — Invulnerable cell at threshold does not split ─────────────

#[test]
fn invulnerable_cell_at_threshold_does_not_split() {
    let mut app = split_test_app();
    let parent = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp {
                current:  20.0,
                starting: 10.0,
                max:      Some(20.0),
            },
            KilledBy::default(),
            crate::shared::death_pipeline::Invulnerable,
        ))
        .id();

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
    assert_eq!(cell_count(&mut app), 1);
}

// ── Behavior 38 — starting == 0.0 does NOT trigger split (guard) ────────────

#[test]
fn zero_starting_cell_does_not_trigger_split() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 0.0, 0.0, Some(0.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 0.0).abs() < f32::EPSILON,
        "zero-starting cell must NOT enter split path; got current {}",
        hp.current
    );
    assert_eq!(cell_count(&mut app), 1, "no new cells from zero-starting");
}

// ════════════════════════════════════════════════════════════════════════════
// Group G — cardinal slot occupancy
// ════════════════════════════════════════════════════════════════════════════

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
        .filter(|(e, ..)| {
            *e != parent
                && app
                    .world()
                    .get::<crate::shared::death_pipeline::Dead>(*e)
                    .is_none()
        })
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

// ── Behavior 44 — spawned cells have full collision suite ───────────────────

#[test]
fn spawned_cells_carry_full_collision_suite() {
    let mut app = split_test_app();
    let _parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    // Query for all new cells with full collision suite.
    let mut q = app.world_mut().query::<(
        &Cell,
        &CellWidth,
        &CellHeight,
        &Aabb2D,
        &CollisionLayers,
        &Hp,
        &KilledBy,
        &Position2D,
    )>();
    // The parent also matches — we'd need to exclude it. Simpler: count all
    // matching cells and assert >= 3 (parent + 2 new). Then filter new cells
    // by position offset.
    let all: Vec<Vec2> = q.iter(app.world()).map(|t| t.7.0).collect();
    let at_offset = all
        .iter()
        .filter(|p| {
            approx_eq_vec2(**p, Vec2::new(70.0, 0.0), 1e-4)
                || approx_eq_vec2(**p, Vec2::new(-70.0, 0.0), 1e-4)
        })
        .count();
    assert_eq!(
        at_offset, 2,
        "2 new cells must match the full collision suite query"
    );
}

// ── Behavior 45 — spawned cells carry CleanupOnExit<NodeState> via require ─

#[test]
fn spawned_cells_carry_cleanup_on_exit_from_cell_require() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let mut q = app
        .world_mut()
        .query::<(Entity, &Cell, &CleanupOnExit<NodeState>)>();
    let count = q.iter(app.world()).filter(|(e, ..)| *e != parent).count();
    assert_eq!(
        count, 2,
        "2 new cells must each carry CleanupOnExit<NodeState> via Cell's #[require]"
    );
}

// ── Behavior 46 — spawned cells carry Scale2D = (70, 24) ────────────────────

#[test]
fn spawned_cells_have_momentum_cell_scale() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let mut q = app
        .world_mut()
        .query::<(Entity, &Scale2D, &Position2D, &Cell)>();
    let mut seen = 0;
    for (e, scale, _pos, _cell) in q.iter(app.world()) {
        if e == parent {
            continue;
        }
        assert!(
            (scale.x - 70.0).abs() < f32::EPSILON,
            "spawned cell scale.x must be 70.0; got {}",
            scale.x
        );
        assert!(
            (scale.y - 24.0).abs() < f32::EPSILON,
            "spawned cell scale.y must be 24.0; got {}",
            scale.y
        );
        seen += 1;
    }
    assert_eq!(seen, 2);
    let _ = MOMENTUM_CELL_WIDTH;
    let _ = MOMENTUM_CELL_HEIGHT;
}

// ── Behavior 47 — multiple cells at threshold split independently ───────────

#[test]
fn multiple_threshold_cells_split_independently() {
    let mut app = split_test_app();
    let a = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));
    let b = spawn_cell_at_with_max(&mut app, Vec2::new(500.0, 500.0), 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp_a = app.world().get::<Hp>(a).unwrap();
    let hp_b = app.world().get::<Hp>(b).unwrap();
    assert!((hp_a.current - 10.0).abs() < f32::EPSILON);
    assert!((hp_b.current - 10.0).abs() < f32::EPSILON);
    // 6 cells total: 2 parents + 4 new (2 each).
    assert_eq!(cell_count(&mut app), 6);
}

// ── Behavior 48 — new cells do NOT re-split on the same tick ────────────────

#[test]
fn new_cells_do_not_split_on_same_tick() {
    let mut app = split_test_app();
    let _parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    // 3 cells: parent + 2 new. If new cells had split on the same tick there'd
    // be more.
    assert_eq!(cell_count(&mut app), 3);

    // Run another tick — still 3 (new cells have current == starting == 10 < threshold 20).
    run_fixed_update(&mut app);
    assert_eq!(cell_count(&mut app), 3);
}

// ── Behavior 49 — register gate blocks split_check when 0 stacks ────────────

#[test]
fn register_gate_blocks_split_when_zero_stacks() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    // 0 stacks.

    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "0-stack gate must block split; hp.current should stay at 20.0; got {}",
        hp.current
    );
    assert_eq!(cell_count(&mut app), 1);
}

#[test]
fn register_gate_positive_control_one_stack_does_split() {
    let mut app = test_app_playing();
    register(&mut app);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);

    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert_eq!(cell_count(&mut app), 3);
}

// ── Behavior 50 — split_check on empty world does not panic ─────────────────

#[test]
fn split_check_empty_world_does_not_panic() {
    let mut app = split_test_app();
    // No cells spawned.
    run_fixed_update(&mut app);
    assert_eq!(cell_count(&mut app), 0);
}

// ════════════════════════════════════════════════════════════════════════════
// Group F addendum — Behavior 71: hp.max preservation on split
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn split_does_not_reset_hp_max_on_parent() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "current resets to starting; got {}",
        hp.current
    );
    assert!(
        (hp.starting - 10.0).abs() < f32::EPSILON,
        "starting is unchanged"
    );
    assert_eq!(
        hp.max,
        Some(20.0),
        "hp.max must be PRESERVED at Some(20.0) after split; got {:?}",
        hp.max
    );
}

#[test]
fn split_preserves_larger_existing_hp_max() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 30.0, 10.0, Some(50.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert_eq!(
        hp.max,
        Some(50.0),
        "larger pre-existing hp.max must be preserved at Some(50.0); got {:?}",
        hp.max
    );
}

#[test]
fn spawned_cells_have_max_none_after_split() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let cells = all_cells(&mut app);
    for (e, _, hp) in &cells {
        if *e != parent {
            assert_eq!(
                hp.max, None,
                "spawned cells must have hp.max = None (ceiling lift is attach_momentum_ceiling's job next tick); got {:?}",
                hp.max
            );
        }
    }
    // Touch imports.
    let _ = CELL_LAYER;
    let _ = BOLT_LAYER;
    let _: MomentumConfig = canonical_momentum_config();
}
