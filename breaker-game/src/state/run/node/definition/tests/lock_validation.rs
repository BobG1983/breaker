//! Lock validation tests for `NodeLayout` — validates the `locks` field against
//! grid bounds, empty-cell positions, and self-referencing locks.

use std::collections::HashMap;

use super::{super::types::*, helpers::test_registry};

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

// ── Part B behavior 26: locks field defaults to None when absent ──

#[test]
fn locks_defaults_to_none_when_absent_in_ron() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: [["S", "."]],
    )"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        layout.locks.is_none(),
        "missing locks field should default to None"
    );
}

#[test]
fn locks_explicit_none_deserializes() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: [["S", "."]],
        locks: None,
    )"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        layout.locks.is_none(),
        "explicit locks: None should produce None"
    );
}

// ── Part B behavior 27: locks field deserializes valid lock mapping ──

#[test]
fn locks_deserializes_valid_lock_mapping() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: [["S", "S"], ["S", "S"]],
        locks: Some({(0, 0): [(0, 1), (1, 0)]}),
    )"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    let locks = layout.locks.as_ref().expect("locks should be Some");
    let targets = locks.get(&(0, 0)).expect("should have key (0, 0)");
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&(0, 1)));
    assert!(targets.contains(&(1, 0)));
}

#[test]
fn locks_deserializes_single_lock_single_target() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 4,
        rows: 4,
        grid_top_offset: 50.0,
        grid: [["S","S","S","S"],["S","S","S","S"],["S","S","S","S"],["S","S","S","S"]],
        locks: Some({(2, 3): [(2, 4)]}),
    )"#;
    // Note: this RON parses but the target (2, 4) is out of bounds for cols=4.
    // We only test deserialization here, not validation.
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(layout.locks.is_some());
}

// ── Part B behavior 28: validate accepts locks within grid bounds ──

#[test]
fn validate_accepts_locks_within_grid_bounds() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(0, 1), (1, 2)]);
    let layout = NodeLayout {
        name: "lock_valid".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "locks within grid bounds should be accepted"
    );
}

#[test]
fn validate_accepts_lock_at_boundary_position() {
    let mut locks = HashMap::new();
    // (1, 2) is the last row, last col for a 2x3 grid (0-indexed)
    locks.insert((1, 2), vec![(0, 0)]);
    let layout = NodeLayout {
        name: "lock_boundary".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "lock at boundary position (1,2) for 2x3 grid should be valid"
    );
}

// ── Part B behavior 29: validate rejects lock key out of grid bounds ──

#[test]
fn validate_rejects_lock_key_row_out_of_bounds() {
    let mut locks = HashMap::new();
    locks.insert((2, 0), vec![(0, 0)]); // row 2 exceeds rows=2 (max index=1)
    let layout = NodeLayout {
        name: "oob_key_row".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock key with row out of bounds should be rejected"
    );
}

#[test]
fn validate_rejects_lock_key_col_out_of_bounds() {
    let mut locks = HashMap::new();
    locks.insert((0, 3), vec![(0, 0)]); // col 3 exceeds cols=3 (max index=2)
    let layout = NodeLayout {
        name: "oob_key_col".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock key with col out of bounds should be rejected"
    );
}

// ── Part B behavior 30: validate rejects lock target out of grid bounds ──

#[test]
fn validate_rejects_lock_target_row_out_of_bounds() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(5, 0)]); // target row 5 exceeds grid
    let layout = NodeLayout {
        name: "oob_target_row".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock target with row out of bounds should be rejected"
    );
}

#[test]
fn validate_rejects_lock_target_col_out_of_bounds() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(0, 10)]); // target col 10 exceeds cols=3
    let layout = NodeLayout {
        name: "oob_target_col".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock target with col out of bounds should be rejected"
    );
}

// ── Part B behavior 31: validate accepts locks: None ─────────────

#[test]
fn validate_accepts_locks_none() {
    let layout = NodeLayout {
        name: "no_locks".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let registry = test_registry();
    assert!(layout.validate(&registry).is_ok());
}

// ── Part B behavior 32: validate accepts empty locks map ─────────

#[test]
fn validate_accepts_empty_locks_map() {
    let layout = NodeLayout {
        name: "empty_locks".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(HashMap::new()),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "empty locks map should be valid"
    );
}

// ── Part B behavior 35: validate rejects lock key on empty cell ──

#[test]
fn validate_rejects_lock_key_pointing_to_empty_cell() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]); // key (0,0) is "." (empty)
    let layout = NodeLayout {
        name: "lock_on_empty".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("."), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock key pointing to empty cell position should be rejected"
    );
}

// ── Part B behavior 36: validate rejects lock target on empty cell ──

#[test]
fn validate_rejects_lock_target_pointing_to_empty_cell() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]); // target (0,1) is "." (empty)
    let layout = NodeLayout {
        name: "lock_target_empty".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s(".")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock target pointing to empty cell position should be rejected"
    );
}

#[test]
fn validate_rejects_lock_with_one_empty_target_among_many() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(1, 0), (0, 1)]); // (0,1) is "." (empty)
    let layout = NodeLayout {
        name: "mixed_targets".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s(".")], vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock with one empty target among valid ones should still be rejected"
    );
}

// ── Part B behavior 37: validate rejects self-referencing lock ───

#[test]
fn validate_rejects_self_referencing_lock() {
    let mut locks = HashMap::new();
    locks.insert((0, 0), vec![(1, 0), (0, 0)]); // target (0,0) == key (0,0)
    let layout = NodeLayout {
        name: "self_ref".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")], vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("self-referencing lock should be rejected");
    assert!(
        err.contains("self") || err.contains("same position"),
        "error should mention self-reference, got: {err}"
    );
}

#[test]
fn validate_rejects_all_targets_equal_key() {
    let mut locks = HashMap::new();
    locks.insert((1, 1), vec![(1, 1)]); // only target equals key
    let layout = NodeLayout {
        name: "all_self".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")], vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: Some(locks),
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_err(),
        "lock where all targets equal the key should be rejected"
    );
}
