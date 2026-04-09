//! Core `NodeLayout` validation tests — valid layouts, unknown aliases, dimension
//! mismatches, cell counting, and grid dimension bounds.

use super::{super::types::*, helpers::test_registry};
use crate::cells::definition::Toughness;

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

#[test]
fn validate_passes_valid_layout() {
    let layout = NodeLayout {
        name: "test".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("T"), s(".")], vec![s("."), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_ok());
}

#[test]
fn validate_rejects_unknown_alias() {
    let layout = NodeLayout {
        name: "bad".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("X"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_err());
}

#[test]
fn validate_rejects_wrong_row_count() {
    let layout = NodeLayout {
        name: "bad".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 3,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_err());
}

#[test]
fn validate_rejects_wrong_col_count() {
    let layout = NodeLayout {
        name: "bad".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_err());
}

#[test]
fn cell_count_skips_dots() {
    let layout = NodeLayout {
        name: "test".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("."), s("T")], vec![s("."), s("S"), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert_eq!(layout.cell_count(), 3);
}

#[test]
fn validate_rejects_cols_above_max() {
    let layout = NodeLayout {
        name: "big_cols".to_owned(),
        timer_secs: 60.0,
        cols: 129,
        rows: 5,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"); 129]; 5],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "cols=129 exceeds MAX_GRID_COLS={MAX_GRID_COLS} and must be rejected",
    );
}

#[test]
fn validate_rejects_rows_above_max() {
    let layout = NodeLayout {
        name: "big_rows".to_owned(),
        timer_secs: 60.0,
        cols: 5,
        rows: 129,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"); 5]; 129],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "rows=129 exceeds MAX_GRID_ROWS={MAX_GRID_ROWS} and must be rejected",
    );
}

#[test]
fn validate_rejects_zero_cols() {
    let layout = NodeLayout {
        name: "zero_cols".to_owned(),
        timer_secs: 60.0,
        cols: 0,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "cols=0 must be rejected",
    );
}

#[test]
fn validate_rejects_zero_rows() {
    let layout = NodeLayout {
        name: "zero_rows".to_owned(),
        timer_secs: 60.0,
        cols: 5,
        rows: 0,
        grid_top_offset: 50.0,
        grid: vec![],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "rows=0 must be rejected",
    );
}

#[test]
fn validate_accepts_max_dimensions() {
    let layout = NodeLayout {
        name: "max_grid".to_owned(),
        timer_secs: 60.0,
        cols: 128,
        rows: 128,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"); 128]; 128],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "128x128 is the maximum valid dimension and must be accepted",
    );
}

#[test]
fn validate_accepts_minimum_dimensions() {
    let layout = NodeLayout {
        name: "tiny".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "1x1 is the minimum valid dimension and must be accepted",
    );
}

// ── Part B: Multi-char String aliases (behaviors 19-25) ──────────

#[test]
fn validate_accepts_single_char_string_aliases() {
    let layout = NodeLayout {
        name: "single_char".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("."), s("T")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_ok());
}

#[test]
fn validate_accepts_all_dot_grid() {
    let layout = NodeLayout {
        name: "all_dots".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("."), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "all-dot grid should be valid"
    );
}

#[test]
fn validate_accepts_multi_char_string_alias() {
    let mut registry = test_registry();
    registry.insert(
        "Gu".to_owned(),
        crate::cells::CellTypeDefinition {
            id: "guard".to_owned(),
            alias: "Gu".to_owned(),
            toughness: Toughness::default(),
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,

            effects: None,
        },
    );
    let layout = NodeLayout {
        name: "multi_char".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("Gu"), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert!(layout.validate(&registry).is_ok());
}

#[test]
fn validate_accepts_three_char_alias_in_registry() {
    let mut registry = test_registry();
    registry.insert(
        "Shd".to_owned(),
        crate::cells::CellTypeDefinition {
            id: "shielded".to_owned(),
            alias: "Shd".to_owned(),
            toughness: Toughness::default(),
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,

            effects: None,
        },
    );
    let layout = NodeLayout {
        name: "three_char".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("Shd")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert!(layout.validate(&registry).is_ok());
}

#[test]
fn validate_rejects_unknown_string_alias() {
    let layout = NodeLayout {
        name: "unknown".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("X"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("unknown alias should be rejected");
    assert!(
        err.contains("unknown alias") && err.contains('X'),
        "error should mention 'unknown alias' and 'X', got: {err}"
    );
}

#[test]
fn validate_rejects_unknown_multi_char_alias() {
    let layout = NodeLayout {
        name: "unknown_multi".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("Xx")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("unknown multi-char alias should be rejected");
    assert!(
        err.contains("Xx"),
        "error should contain the unknown alias 'Xx', got: {err}"
    );
}

#[test]
fn cell_count_with_string_grid() {
    let layout = NodeLayout {
        name: "count".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("."), s("T")], vec![s("."), s("S"), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert_eq!(layout.cell_count(), 3);
}

#[test]
fn cell_count_all_dots_returns_zero() {
    let layout = NodeLayout {
        name: "empty".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("."), s(".")], vec![s("."), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert_eq!(layout.cell_count(), 0);
}

#[test]
fn cell_count_no_dots_returns_total() {
    let layout = NodeLayout {
        name: "full".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("T")], vec![s("S"), s("T")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    assert_eq!(layout.cell_count(), 4);
}

#[test]
fn validate_still_checks_row_count_against_declared_rows() {
    let layout = NodeLayout {
        name: "mismatch".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 3,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")], vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_err());
}

#[test]
fn validate_still_checks_col_count_per_row() {
    let layout = NodeLayout {
        name: "jagged".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("jagged grid should be rejected");
    assert!(
        err.contains("row 1"),
        "error should mention row 1, got: {err}"
    );
}
