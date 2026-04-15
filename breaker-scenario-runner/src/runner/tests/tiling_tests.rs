//! Tests for `grid_dimensions`, `tile_position`, environment variable constants,
//! `TileConfig`, `tile_config_env_vars`, and `parse_tile_config`.

use bevy::prelude::*;

use crate::runner::tiling::{
    DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH, ENV_TILE_COUNT, ENV_TILE_INDEX, TileConfig,
    TilePosition, grid_dimensions, parse_tile_config, tile_config_env_vars, tile_position,
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

// =========================================================================
// DEFAULT_SCREEN_WIDTH / DEFAULT_SCREEN_HEIGHT — constants
// =========================================================================

#[test]
fn default_screen_size_constants_have_correct_values() {
    assert_eq!(DEFAULT_SCREEN_WIDTH, 1920);
    assert_eq!(DEFAULT_SCREEN_HEIGHT, 1080);
}

// =========================================================================
// Spec Behavior 1: New env var constants have correct string values
// =========================================================================

#[test]
fn env_tile_index_constant_has_correct_value() {
    assert_eq!(
        ENV_TILE_INDEX, "SCENARIO_TILE_INDEX",
        "ENV_TILE_INDEX must be 'SCENARIO_TILE_INDEX'"
    );
}

#[test]
fn env_tile_count_constant_has_correct_value() {
    assert_eq!(
        ENV_TILE_COUNT, "SCENARIO_TILE_COUNT",
        "ENV_TILE_COUNT must be 'SCENARIO_TILE_COUNT'"
    );
}

// =========================================================================
// Spec Behavior 2: tile_config_env_vars returns index and count for slot 0/4
// =========================================================================

#[test]
fn tile_config_env_vars_slot_0_of_4_returns_index_and_count_pairs() {
    let pairs = tile_config_env_vars(0, 4);
    assert_eq!(pairs.len(), 2, "expected exactly 2 env var pairs");
    assert_eq!(
        pairs[0],
        ("SCENARIO_TILE_INDEX", "0".to_owned()),
        "first pair must be (SCENARIO_TILE_INDEX, '0')"
    );
    assert_eq!(
        pairs[1],
        ("SCENARIO_TILE_COUNT", "4".to_owned()),
        "second pair must be (SCENARIO_TILE_COUNT, '4')"
    );
}

// =========================================================================
// Spec Behavior 3: tile_config_env_vars correct for slot 3/4 (last slot)
// =========================================================================

#[test]
fn tile_config_env_vars_slot_3_of_4_returns_correct_pairs() {
    let pairs = tile_config_env_vars(3, 4);
    assert_eq!(pairs.len(), 2, "expected exactly 2 env var pairs");
    assert_eq!(
        pairs[0],
        ("SCENARIO_TILE_INDEX", "3".to_owned()),
        "first pair must be (SCENARIO_TILE_INDEX, '3')"
    );
    assert_eq!(
        pairs[1],
        ("SCENARIO_TILE_COUNT", "4".to_owned()),
        "second pair must be (SCENARIO_TILE_COUNT, '4')"
    );
}

// =========================================================================
// Spec Behavior 4: tile_config_env_vars correct for single scenario
// =========================================================================

#[test]
fn tile_config_env_vars_single_scenario_returns_correct_pairs() {
    let pairs = tile_config_env_vars(0, 1);
    assert_eq!(pairs.len(), 2, "expected exactly 2 env var pairs");
    assert_eq!(
        pairs[0],
        ("SCENARIO_TILE_INDEX", "0".to_owned()),
        "first pair must be (SCENARIO_TILE_INDEX, '0')"
    );
    assert_eq!(
        pairs[1],
        ("SCENARIO_TILE_COUNT", "1".to_owned()),
        "second pair must be (SCENARIO_TILE_COUNT, '1')"
    );
}

// =========================================================================
// Spec Behavior 5: tile_config_env_vars correct for large total
// =========================================================================

#[test]
fn tile_config_env_vars_large_total_returns_correct_pairs() {
    let pairs = tile_config_env_vars(99, 100);
    assert_eq!(pairs.len(), 2, "expected exactly 2 env var pairs");
    assert_eq!(
        pairs[0],
        ("SCENARIO_TILE_INDEX", "99".to_owned()),
        "first pair must be (SCENARIO_TILE_INDEX, '99')"
    );
    assert_eq!(
        pairs[1],
        ("SCENARIO_TILE_COUNT", "100".to_owned()),
        "second pair must be (SCENARIO_TILE_COUNT, '100')"
    );
}

// =========================================================================
// Spec Behavior 6: TileConfig can be constructed with index and count
// =========================================================================

#[test]
fn tile_config_construction_with_index_and_count() {
    let config = TileConfig { index: 2, count: 9 };
    assert_eq!(config.index, 2);
    assert_eq!(config.count, 9);
}

#[test]
fn tile_config_construction_single_scenario() {
    let config = TileConfig { index: 0, count: 1 };
    assert_eq!(config.index, 0);
    assert_eq!(config.count, 1);
}

// =========================================================================
// Spec Behavior 7: TileConfig is a Bevy Resource
// =========================================================================

#[test]
fn tile_config_is_a_bevy_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TileConfig { index: 0, count: 4 });

    let config = app.world().resource::<TileConfig>();
    assert_eq!(config.index, 0);
    assert_eq!(config.count, 4);
}

// =========================================================================
// Spec Behavior 8: parse_tile_config returns None when getter returns None
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_no_env_vars_set() {
    let result = parse_tile_config(|_| None);
    assert_eq!(result, None);
}

// =========================================================================
// Spec Behavior 9: parse_tile_config returns Some when both keys are valid
// =========================================================================

#[test]
fn parse_tile_config_returns_some_when_both_keys_valid() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("2".to_owned()),
        "SCENARIO_TILE_COUNT" => Some("9".to_owned()),
        _ => None,
    });
    assert!(
        result.is_some(),
        "expected Some(TileConfig) when both keys are valid"
    );
    let config = result.unwrap();
    assert_eq!(config.index, 2);
    assert_eq!(config.count, 9);
}

#[test]
fn parse_tile_config_returns_some_for_boundary_values() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("0".to_owned()),
        "SCENARIO_TILE_COUNT" => Some("1".to_owned()),
        _ => None,
    });
    assert!(
        result.is_some(),
        "expected Some(TileConfig) for index=0, count=1"
    );
    let config = result.unwrap();
    assert_eq!(config.index, 0);
    assert_eq!(config.count, 1);
}

// =========================================================================
// Spec Behavior 10: parse_tile_config returns None when only index present
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_only_index_present() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("3".to_owned()),
        _ => None,
    });
    assert_eq!(result, None, "expected None when only index is present");
}

// =========================================================================
// Spec Behavior 11: parse_tile_config returns None when only count present
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_only_count_present() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_COUNT" => Some("4".to_owned()),
        _ => None,
    });
    assert_eq!(result, None, "expected None when only count is present");
}

// =========================================================================
// Spec Behavior 12: parse_tile_config returns None when index non-numeric
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_index_non_numeric() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("abc".to_owned()),
        "SCENARIO_TILE_COUNT" => Some("4".to_owned()),
        _ => None,
    });
    assert_eq!(result, None, "expected None when index is non-numeric");
}

// =========================================================================
// Spec Behavior 13: parse_tile_config returns None when index negative
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_index_negative() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("-1".to_owned()),
        "SCENARIO_TILE_COUNT" => Some("4".to_owned()),
        _ => None,
    });
    assert_eq!(
        result, None,
        "expected None when index is negative (cannot parse as u32)"
    );
}

// =========================================================================
// Spec Behavior 14: parse_tile_config returns None when count is zero
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_count_is_zero() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" | "SCENARIO_TILE_COUNT" => Some("0".to_owned()),
        _ => None,
    });
    assert_eq!(
        result, None,
        "expected None when count is zero (no scenarios to tile)"
    );
}

// =========================================================================
// Spec Behavior 15: parse_tile_config returns None when count non-numeric
// =========================================================================

#[test]
fn parse_tile_config_returns_none_when_count_non_numeric() {
    let result = parse_tile_config(|key| match key {
        "SCENARIO_TILE_INDEX" => Some("0".to_owned()),
        "SCENARIO_TILE_COUNT" => Some("xyz".to_owned()),
        _ => None,
    });
    assert_eq!(result, None, "expected None when count is non-numeric");
}
