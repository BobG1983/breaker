//! Tests for `grid_dimensions`, `tile_position`, and environment variable constants.

use bevy::prelude::*;

use crate::runner::tiling::{
    DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH, ENV_WINDOW_H, ENV_WINDOW_W, ENV_WINDOW_X,
    ENV_WINDOW_Y, TilePosition, grid_dimensions, parse_tile_env, tile_env_vars, tile_position,
    window_from_tile,
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

// =========================================================================
// DEFAULT_SCREEN_WIDTH / DEFAULT_SCREEN_HEIGHT — constants
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 17: default screen size constants have correct values
// -------------------------------------------------------------------------

#[test]
fn default_screen_size_constants_have_correct_values() {
    assert_eq!(DEFAULT_SCREEN_WIDTH, 1920);
    assert_eq!(DEFAULT_SCREEN_HEIGHT, 1080);
}

// =========================================================================
// tile_env_vars — pure function computing env var key-value pairs
// =========================================================================

/// Helper: converts a `Vec<(&str, String)>` to a sorted list of `(key, value)`
/// pairs for order-independent comparison.
fn sorted_env_pairs(pairs: Vec<(&str, String)>) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

/// Helper: builds sorted expected pairs from four concrete values.
fn expected_env_pairs(
    pos_x: &str,
    pos_y: &str,
    width: &str,
    height: &str,
) -> Vec<(String, String)> {
    let mut v = vec![
        ("SCENARIO_WINDOW_X".to_owned(), pos_x.to_owned()),
        ("SCENARIO_WINDOW_Y".to_owned(), pos_y.to_owned()),
        ("SCENARIO_WINDOW_W".to_owned(), width.to_owned()),
        ("SCENARIO_WINDOW_H".to_owned(), height.to_owned()),
    ];
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

// -------------------------------------------------------------------------
// Behavior 1: tile_env_vars returns correct pairs for slot 0 of 4
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_slot_0_of_4_returns_top_left_half_screen() {
    let pairs = tile_env_vars(0, 4);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("0", "0", "960", "540"),
    );
}

// -------------------------------------------------------------------------
// Behavior 2: tile_env_vars returns correct pairs for slot 3 of 4
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_slot_3_of_4_returns_bottom_right_half_screen() {
    let pairs = tile_env_vars(3, 4);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("960", "540", "960", "540"),
    );
}

// -------------------------------------------------------------------------
// Behavior 3: tile_env_vars returns correct pairs for slot 2 of 5 (3x2)
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_slot_2_of_5_returns_third_column_first_row() {
    let pairs = tile_env_vars(2, 5);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("1280", "0", "640", "540"),
    );
}

// -------------------------------------------------------------------------
// Behavior 4: tile_env_vars returns full-screen pairs for single scenario
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_single_scenario_returns_full_screen() {
    let pairs = tile_env_vars(0, 1);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("0", "0", "1920", "1080"),
    );
}

// -------------------------------------------------------------------------
// Behavior 5: tile_env_vars returns correct pairs for slot 0 of 2 (2x1)
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_slot_0_of_2_returns_left_half_full_height() {
    let pairs = tile_env_vars(0, 2);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("0", "0", "960", "1080"),
    );
}

// -------------------------------------------------------------------------
// Behavior 6: tile_env_vars with zero total does not panic
// -------------------------------------------------------------------------

#[test]
fn tile_env_vars_zero_total_does_not_panic_and_returns_full_screen() {
    let pairs = tile_env_vars(0, 0);
    assert_eq!(pairs.len(), 4, "expected 4 env var pairs");
    assert_eq!(
        sorted_env_pairs(pairs),
        expected_env_pairs("0", "0", "1920", "1080"),
    );
}

// =========================================================================
// parse_tile_env — pure function with dependency-injected getter
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 7: parse_tile_env returns None when getter returns None for all
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_none_when_no_env_vars_set() {
    let result = parse_tile_env(|_| None);
    assert_eq!(result, None);
}

// -------------------------------------------------------------------------
// Behavior 8: parse_tile_env returns Some when all four keys are valid
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_tile_position_when_all_four_keys_valid() {
    let result = parse_tile_env(|key| match key {
        "SCENARIO_WINDOW_X" | "SCENARIO_WINDOW_W" => Some("960".to_owned()),
        "SCENARIO_WINDOW_Y" | "SCENARIO_WINDOW_H" => Some("540".to_owned()),
        _ => None,
    });
    assert_eq!(
        result,
        Some(TilePosition {
            x: 960,
            y: 540,
            width: 960,
            height: 540,
        }),
    );
}

// -------------------------------------------------------------------------
// Behavior 9: parse_tile_env returns None when only some keys present
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_none_when_only_x_and_y_present() {
    let result = parse_tile_env(|key| match key {
        "SCENARIO_WINDOW_X" | "SCENARIO_WINDOW_Y" => Some("0".to_owned()),
        _ => None,
    });
    assert_eq!(result, None);
}

// -------------------------------------------------------------------------
// Behavior 10: parse_tile_env returns None when a value is non-numeric
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_none_when_x_is_non_numeric() {
    let result = parse_tile_env(|key| match key {
        "SCENARIO_WINDOW_X" => Some("abc".to_owned()),
        "SCENARIO_WINDOW_Y" => Some("0".to_owned()),
        "SCENARIO_WINDOW_W" => Some("960".to_owned()),
        "SCENARIO_WINDOW_H" => Some("540".to_owned()),
        _ => None,
    });
    assert_eq!(result, None);
}

// -------------------------------------------------------------------------
// Behavior 11: parse_tile_env returns None when a value is negative
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_none_when_x_is_negative() {
    let result = parse_tile_env(|key| match key {
        "SCENARIO_WINDOW_X" => Some("-1".to_owned()),
        "SCENARIO_WINDOW_Y" => Some("0".to_owned()),
        "SCENARIO_WINDOW_W" => Some("960".to_owned()),
        "SCENARIO_WINDOW_H" => Some("540".to_owned()),
        _ => None,
    });
    assert_eq!(result, None);
}

// -------------------------------------------------------------------------
// Behavior 12: parse_tile_env returns Some for full-screen tile values
// -------------------------------------------------------------------------

#[test]
fn parse_tile_env_returns_full_screen_tile_when_all_values_full_hd() {
    let result = parse_tile_env(|key| match key {
        "SCENARIO_WINDOW_X" | "SCENARIO_WINDOW_Y" => Some("0".to_owned()),
        "SCENARIO_WINDOW_W" => Some("1920".to_owned()),
        "SCENARIO_WINDOW_H" => Some("1080".to_owned()),
        _ => None,
    });
    assert_eq!(
        result,
        Some(TilePosition {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }),
    );
}

// =========================================================================
// window_from_tile — pure function converting TilePosition to Window
// =========================================================================

// -------------------------------------------------------------------------
// Behavior 13: window_from_tile produces correct position and resolution
// -------------------------------------------------------------------------

#[test]
fn window_from_tile_produces_correct_position_and_resolution_for_tiled_window() {
    let tile = TilePosition {
        x: 640,
        y: 0,
        width: 640,
        height: 540,
    };
    let window = window_from_tile(&tile);

    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(640, 0)),
        "window position should match tile x,y"
    );
    assert!(
        (window.resolution.width() - 640.0).abs() < f32::EPSILON,
        "window width should be 640.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 540.0).abs() < f32::EPSILON,
        "window height should be 540.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Behavior 14: window_from_tile produces correct fields for full-screen
// -------------------------------------------------------------------------

#[test]
fn window_from_tile_produces_correct_fields_for_full_screen_tile() {
    let tile = TilePosition {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    let window = window_from_tile(&tile);

    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(0, 0)),
        "window position should be at origin"
    );
    assert!(
        (window.resolution.width() - 1920.0).abs() < f32::EPSILON,
        "window width should be 1920.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 1080.0).abs() < f32::EPSILON,
        "window height should be 1080.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Behavior 15: window_from_tile sets the title to "Scenario Runner"
// -------------------------------------------------------------------------

#[test]
fn window_from_tile_sets_title_to_scenario_runner() {
    let tile = TilePosition {
        x: 0,
        y: 0,
        width: 960,
        height: 540,
    };
    let window = window_from_tile(&tile);

    assert_eq!(
        window.title, "Scenario Runner",
        "window title should be 'Scenario Runner'"
    );
}

// -------------------------------------------------------------------------
// Behavior 16: window_from_tile produces correct fields for bottom-right 2x2
// -------------------------------------------------------------------------

#[test]
fn window_from_tile_produces_correct_fields_for_bottom_right_in_2x2() {
    let tile = TilePosition {
        x: 960,
        y: 540,
        width: 960,
        height: 540,
    };
    let window = window_from_tile(&tile);

    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(960, 540)),
        "window position should match bottom-right tile"
    );
    assert!(
        (window.resolution.width() - 960.0).abs() < f32::EPSILON,
        "window width should be 960.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 540.0).abs() < f32::EPSILON,
        "window height should be 540.0, got {}",
        window.resolution.height()
    );
}
