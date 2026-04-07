//! Tests for `grid_dimensions`, `tile_position`, and environment variable constants.

use crate::runner::tiling::{
    ENV_WINDOW_H, ENV_WINDOW_W, ENV_WINDOW_X, ENV_WINDOW_Y, TilePosition, grid_dimensions,
    tile_position,
};

// =========================================================================
// grid_dimensions — pure function returning (cols, rows) for n scenarios
// =========================================================================

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

// =========================================================================
// tile_position — pure function returning TilePosition for a grid slot
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 10: top-left tile in 2x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_top_left_in_2x2_on_1920x1080() {
    let pos = tile_position(0, 2, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 0,
            y: 0,
            width: 960,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 11: top-right tile in 2x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_top_right_in_2x2_on_1920x1080() {
    let pos = tile_position(1, 2, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 960,
            y: 0,
            width: 960,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 12: bottom-left tile in 2x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_bottom_left_in_2x2_on_1920x1080() {
    let pos = tile_position(2, 2, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 0,
            y: 540,
            width: 960,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 13: bottom-right tile in 2x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_bottom_right_in_2x2_on_1920x1080() {
    let pos = tile_position(3, 2, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 960,
            y: 540,
            width: 960,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 14: single tile fills entire screen
// -------------------------------------------------------------------------

#[test]
fn tile_position_single_tile_fills_entire_screen() {
    let pos = tile_position(0, 1, 1, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 15: middle of first row in 3x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_middle_first_row_in_3x2_on_1920x1080() {
    let pos = tile_position(1, 3, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 640,
            y: 0,
            width: 640,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 16: first slot of second row in 3x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_first_slot_second_row_in_3x2_on_1920x1080() {
    let pos = tile_position(3, 3, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 0,
            y: 540,
            width: 640,
            height: 540,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 17: non-evenly-divisible screen uses integer truncation
// -------------------------------------------------------------------------

#[test]
fn tile_position_non_divisible_screen_uses_integer_truncation() {
    let pos = tile_position(0, 3, 3, 1000, 1000);
    assert_eq!(
        pos,
        TilePosition {
            x: 0,
            y: 0,
            width: 333,
            height: 333,
        }
    );
}

// -------------------------------------------------------------------------
// Behavior 18: last slot in non-full grid (slot 4 in 3x2)
// -------------------------------------------------------------------------

#[test]
fn tile_position_last_slot_in_non_full_3x2_grid() {
    let pos = tile_position(4, 3, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x: 640,
            y: 540,
            width: 640,
            height: 540,
        }
    );
}

// =========================================================================
// Environment variable constants
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 19: env var constants have the correct string values
// -------------------------------------------------------------------------

#[test]
fn env_var_constants_have_correct_values() {
    assert_eq!(ENV_WINDOW_X, "SCENARIO_WINDOW_X");
    assert_eq!(ENV_WINDOW_Y, "SCENARIO_WINDOW_Y");
    assert_eq!(ENV_WINDOW_W, "SCENARIO_WINDOW_W");
    assert_eq!(ENV_WINDOW_H, "SCENARIO_WINDOW_H");
}
