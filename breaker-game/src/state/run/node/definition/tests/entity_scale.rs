//! Entity scale deserialization and validation boundary tests.

use super::{super::types::*, helpers::test_registry};

#[test]
fn deserialize_default_entity_scale_is_one() {
    let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [["S","S"]])"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        (layout.entity_scale - 1.0).abs() < f32::EPSILON,
        "entity_scale should default to 1.0, got {}",
        layout.entity_scale,
    );
}

#[test]
fn deserialize_explicit_entity_scale_one() {
    let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [["S","S"]], entity_scale: 1.0)"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        (layout.entity_scale - 1.0).abs() < f32::EPSILON,
        "explicit entity_scale: 1.0 should deserialize to 1.0, got {}",
        layout.entity_scale,
    );
}

#[test]
fn deserialize_explicit_entity_scale_custom() {
    let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [["S","S"]], entity_scale: 0.7)"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        (layout.entity_scale - 0.7).abs() < f32::EPSILON,
        "entity_scale: 0.7 should deserialize to 0.7, got {}",
        layout.entity_scale,
    );
}

#[test]
fn deserialize_entity_scale_at_minimum() {
    let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [["S","S"]], entity_scale: 0.5)"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert!(
        (layout.entity_scale - 0.5).abs() < f32::EPSILON,
        "entity_scale: 0.5 should deserialize to 0.5, got {}",
        layout.entity_scale,
    );
}

fn s(val: &str) -> String {
    val.to_owned()
}

#[test]
fn validate_rejects_entity_scale_below_minimum() {
    let layout = NodeLayout {
        name: "scale_low".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 0.49,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("entity_scale 0.49 is below MIN_ENTITY_SCALE and must be rejected");
    assert!(
        err.contains("entity_scale"),
        "error message should mention entity_scale, got: {err}",
    );
}

#[test]
fn validate_rejects_entity_scale_zero() {
    let layout = NodeLayout {
        name: "scale_zero".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 0.0,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("entity_scale 0.0 is below MIN_ENTITY_SCALE and must be rejected");
    assert!(
        err.contains("entity_scale"),
        "error message should mention entity_scale, got: {err}",
    );
}

#[test]
fn validate_rejects_entity_scale_above_maximum() {
    let layout = NodeLayout {
        name: "scale_high".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.01,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("entity_scale 1.01 is above MAX_ENTITY_SCALE and must be rejected");
    assert!(
        err.contains("entity_scale"),
        "error message should mention entity_scale, got: {err}",
    );
}

#[test]
fn validate_rejects_entity_scale_far_above_maximum() {
    let layout = NodeLayout {
        name: "scale_double".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 2.0,
        locks: None,
    };
    let registry = test_registry();
    let err = layout
        .validate(&registry)
        .expect_err("entity_scale 2.0 is above MAX_ENTITY_SCALE and must be rejected");
    assert!(
        err.contains("entity_scale"),
        "error message should mention entity_scale, got: {err}",
    );
}

#[test]
fn validate_accepts_entity_scale_at_minimum() {
    let layout = NodeLayout {
        name: "scale_min".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S"), s("S")]],
        pool: NodePool::default(),
        entity_scale: 0.5,
        locks: None,
    };
    let registry = test_registry();
    assert!(
        layout.validate(&registry).is_ok(),
        "entity_scale 0.5 equals MIN_ENTITY_SCALE and must be accepted",
    );
}

#[test]
fn validate_accepts_entity_scale_at_maximum() {
    let layout = NodeLayout {
        name: "scale_max".to_owned(),
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
    assert!(
        layout.validate(&registry).is_ok(),
        "entity_scale 1.0 equals MAX_ENTITY_SCALE and must be accepted",
    );
}
