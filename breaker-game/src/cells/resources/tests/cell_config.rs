use bevy::prelude::*;

use super::super::data::*;
use crate::cells::definition::{CellTypeDefinition, Toughness};

#[test]
fn cell_defaults_width_height_positive() {
    let config = CellConfig::default();
    assert!(config.width > 0.0);
    assert!(config.height > 0.0);
}

#[test]
fn cell_defaults_ron_parses() {
    let ron_str = include_str!("../../../../assets/config/defaults.cells.ron");
    let result: CellDefaults = ron::de::from_str(ron_str).expect("cells RON should parse");
    assert!(result.width > 0.0);
}

#[test]
fn all_cell_type_rons_parse() {
    use std::fs;
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cells");
    for entry in fs::read_dir(dir).expect("assets/cells/ should exist") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let content = fs::read_to_string(&path).unwrap();
            let def: CellTypeDefinition = ron::de::from_str(&content).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
            assert!(
                !def.id.is_empty(),
                "{}: id must not be empty",
                path.display()
            );
            assert!(
                !def.alias.is_empty(),
                "{}: alias must not be empty",
                path.display()
            );
            assert!(
                def.alias != ".",
                "{}: alias must not be '.'",
                path.display()
            );
            // Toughness is an enum — always valid
            let _ = def.toughness;
            assert!(
                def.validate().is_ok(),
                "{}: validate() should pass: {:?}",
                path.display(),
                def.validate(),
            );
        }
    }
}

fn make_cell_type(id: &str, alias: &str, toughness: Toughness) -> CellTypeDefinition {
    CellTypeDefinition {
        id: id.to_owned(),
        alias: alias.to_owned(),
        toughness,
        color_rgb: [1.0, 1.0, 1.0],
        required_to_clear: true,
        damage_hdr_base: 1.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.5,
        damage_blue_base: 0.2,
        behaviors: None,
        effects: None,
    }
}

#[test]
fn registry_detects_duplicate_aliases() {
    let def_a = make_cell_type("a", "S", Toughness::Standard);
    let def_b = make_cell_type("b", "S", Toughness::Tough);
    let mut registry = CellTypeRegistry::default();
    registry.insert(def_a.alias.clone(), def_a);
    let had_existing = registry.insert(def_b.alias.clone(), def_b);
    assert!(
        had_existing.is_some(),
        "inserting duplicate alias should replace"
    );
}
