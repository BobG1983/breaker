//! Tests for `parse_tile_config` — env var getter → `Option<TileConfig>`.

use crate::runner::tiling::parse_tile_config;

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
