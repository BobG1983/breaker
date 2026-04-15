//! Tests for `tile_position` — pure function returning `TilePosition` for a grid slot.

use crate::runner::tiling::{TilePosition, tile_position};

// -------------------------------------------------------------------------
// Behavior 10: top-left tile in 2x2 grid on 1920x1080
// -------------------------------------------------------------------------

#[test]
fn tile_position_top_left_in_2x2_on_1920x1080() {
    let pos = tile_position(0, 2, 2, 1920, 1080);
    assert_eq!(
        pos,
        TilePosition {
            x:      0,
            y:      0,
            width:  960,
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
            x:      960,
            y:      0,
            width:  960,
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
            x:      0,
            y:      540,
            width:  960,
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
            x:      960,
            y:      540,
            width:  960,
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
            x:      0,
            y:      0,
            width:  1920,
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
            x:      640,
            y:      0,
            width:  640,
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
            x:      0,
            y:      540,
            width:  640,
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
            x:      0,
            y:      0,
            width:  333,
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
            x:      640,
            y:      540,
            width:  640,
            height: 540,
        }
    );
}
