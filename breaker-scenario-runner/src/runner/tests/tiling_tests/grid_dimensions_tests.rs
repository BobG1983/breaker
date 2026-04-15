//! Tests for `grid_dimensions` — pure function returning (cols, rows) for n scenarios.

use crate::runner::tiling::grid_dimensions;

// -------------------------------------------------------------------------
// Behavior 1: single scenario produces a 1x1 grid
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_single_scenario_produces_1x1() {
    let (cols, rows) = grid_dimensions(1);
    assert_eq!((cols, rows), (1, 1));
}

// -------------------------------------------------------------------------
// Behavior 1 edge case: n=0 returns (1,1) — degenerate, must not panic
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_zero_returns_1x1_without_panic() {
    let (cols, rows) = grid_dimensions(0);
    assert_eq!((cols, rows), (1, 1));
}

// -------------------------------------------------------------------------
// Behavior 2: two scenarios produce a 2x1 grid
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_two_scenarios_produces_2x1() {
    let (cols, rows) = grid_dimensions(2);
    assert_eq!((cols, rows), (2, 1));
}

// -------------------------------------------------------------------------
// Behavior 3: three scenarios produce a 2x2 grid
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_three_scenarios_produces_2x2() {
    let (cols, rows) = grid_dimensions(3);
    assert_eq!((cols, rows), (2, 2));
}

// -------------------------------------------------------------------------
// Behavior 4: four scenarios produce a 2x2 grid (perfect square)
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_four_scenarios_produces_2x2() {
    let (cols, rows) = grid_dimensions(4);
    assert_eq!((cols, rows), (2, 2));
}

// -------------------------------------------------------------------------
// Behavior 5: five scenarios produce a 3x2 grid
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_five_scenarios_produces_3x2() {
    let (cols, rows) = grid_dimensions(5);
    assert_eq!((cols, rows), (3, 2));
}

// -------------------------------------------------------------------------
// Behavior 6: nine scenarios produce a 3x3 grid (perfect square)
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_nine_scenarios_produces_3x3() {
    let (cols, rows) = grid_dimensions(9);
    assert_eq!((cols, rows), (3, 3));
}

// -------------------------------------------------------------------------
// Behavior 7: ten scenarios produce a 4x3 grid
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_ten_scenarios_produces_4x3() {
    let (cols, rows) = grid_dimensions(10);
    assert_eq!((cols, rows), (4, 3));
}

// -------------------------------------------------------------------------
// Behavior 8: 100 scenarios produce a 10x10 grid (large perfect square)
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_100_scenarios_produces_10x10() {
    let (cols, rows) = grid_dimensions(100);
    assert_eq!((cols, rows), (10, 10));
}

// -------------------------------------------------------------------------
// Behavior 9: grid always has enough slots for all scenarios (property)
// -------------------------------------------------------------------------

#[test]
fn grid_dimensions_always_has_enough_slots_for_1_through_50() {
    for n in 1..=50 {
        let (cols, rows) = grid_dimensions(n);
        assert!(
            cols * rows >= n,
            "grid_dimensions({n}) = ({cols}, {rows}) has only {} slots, need at least {n}",
            cols * rows,
        );
    }
}
