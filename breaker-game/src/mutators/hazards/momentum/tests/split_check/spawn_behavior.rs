//! Spawned cell properties and run-if gating (Behaviors 44–50).

use bevy::prelude::*;

use super::{
    super::{
        super::system::{MOMENTUM_CELL_HEIGHT, MOMENTUM_CELL_WIDTH, wire},
        helpers::{
            add_momentum_stacks, approx_eq_vec2, canonical_momentum_config, cell_count,
            install_momentum_config, run_fixed_update, spawn_cell_at_with_max, test_app_playing,
        },
    },
    helpers::split_test_app,
};
use crate::{
    cells::components::{CellHeight, CellWidth},
    prelude::*,
};

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

// ── Behavior 49 — wire gate blocks split_check when 0 stacks ────────────

#[test]
fn register_gate_blocks_split_when_zero_stacks() {
    let mut app = test_app_playing();
    wire(&mut app);
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
    wire(&mut app);
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
