//! RON file parsing integration test — validates all `.ron` files in `assets/nodes/`.

use super::{super::types::*, helpers::test_registry};

// ── Part B behavior 33: NodeLayout RON with String grid deserializes ──

#[test]
fn node_layout_ron_with_string_grid_deserializes() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: [["S", "."]],
    )"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(layout.grid[0][0], "S");
    assert_eq!(layout.grid[0][1], ".");
    assert!(layout.locks.is_none(), "locks should default to None");
}

#[test]
fn node_layout_ron_with_multi_char_alias_deserializes() {
    let ron_str = r#"(
        name: "test",
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: [["Gu", "."]],
    )"#;
    let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
    assert_eq!(layout.grid[0][0], "Gu");
}

// ── Part B behavior 34 / Part E behavior 49: All existing node RON files parse ──

#[test]
fn all_node_rons_parse() {
    use std::fs;
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/nodes");
    for entry in fs::read_dir(dir).expect("assets/nodes/ should exist") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let content = fs::read_to_string(&path).unwrap();
            let layout: NodeLayout = ron::de::from_str(&content).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
            let registry = test_registry();
            layout.validate(&registry).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
        }
    }
}
