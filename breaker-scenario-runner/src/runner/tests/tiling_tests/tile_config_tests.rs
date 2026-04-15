//! Tests for `TileConfig`, `DEFAULT_SCREEN_*` constants, env var constants, and
//! `tile_config_env_vars`.

use bevy::prelude::*;

use crate::runner::tiling::{
    DEFAULT_SCREEN_HEIGHT, DEFAULT_SCREEN_WIDTH, ENV_TILE_COUNT, ENV_TILE_INDEX, TileConfig,
    tile_config_env_vars,
};

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
