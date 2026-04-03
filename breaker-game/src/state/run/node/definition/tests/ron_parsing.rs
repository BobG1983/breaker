//! RON file parsing integration test — validates all `.ron` files in `assets/nodes/`.

use super::{super::types::*, helpers::test_registry};

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
