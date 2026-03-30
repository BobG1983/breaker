//! Core `NodeLayout` validation tests — valid layouts, unknown aliases, dimension
//! mismatches, cell counting, and grid dimension bounds.

use super::{super::types::*, helpers::test_registry};

#[test]
fn validate_passes_valid_layout() {
    let layout = NodeLayout {
        name: "test".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec!['S', 'T', '.'], vec!['.', 'S', 'S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['X', 'S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S', 'S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S', '.', 'T'], vec!['.', 'S', '.']],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S'; 129]; 5],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S'; 5]; 129],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S'; 128]; 128],
        pool: NodePool::default(),
        entity_scale: 1.0,
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
        grid: vec![vec!['S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "1x1 is the minimum valid dimension and must be accepted",
    );
}
