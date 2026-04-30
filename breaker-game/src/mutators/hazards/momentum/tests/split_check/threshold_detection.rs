//! Group F — threshold detection (Behaviors 32–38).

use bevy::prelude::*;

use super::{
    super::helpers::{
        all_cells, approx_eq_vec2, cell_count, run_fixed_update, spawn_cell_at_with_max,
    },
    helpers::split_test_app,
};
use crate::prelude::*;

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
            KilledBy { killer: None },
            crate::prelude::Dead,
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
            KilledBy { killer: None },
            crate::prelude::Invulnerable,
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
