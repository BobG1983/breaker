use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::data::*;
use crate::cells::definition::{CellBehavior, CellTypeDefinition};

#[test]
fn cell_defaults_width_height_positive() {
    let config = CellConfig::default();
    assert!(config.width > 0.0);
    assert!(config.height > 0.0);
}

#[test]
fn cell_defaults_ron_parses() {
    let ron_str = include_str!("../../../assets/config/defaults.cells.ron");
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
            assert!(def.hp > 0.0, "{}: hp must be > 0.0", path.display());
            assert!(
                def.validate().is_ok(),
                "{}: validate() should pass: {:?}",
                path.display(),
                def.validate(),
            );
        }
    }
}

#[test]
fn registry_detects_duplicate_aliases() {
    let def_a = make_cell_type("a", "S", 1.0);
    let def_b = make_cell_type("b", "S", 2.0);
    let mut registry = CellTypeRegistry::default();
    registry.insert(def_a.alias.clone(), def_a);
    let had_existing = registry.insert(def_b.alias.clone(), def_b);
    assert!(
        had_existing.is_some(),
        "inserting duplicate alias should replace"
    );
}

// ── SeedableRegistry tests ─────────────────────────────────────

fn make_cell_type(id: &str, alias: &str, hp: f32) -> CellTypeDefinition {
    CellTypeDefinition {
        id: id.to_owned(),
        alias: alias.to_owned(),
        hp,
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

/// Creates `AssetId` values by adding assets to an `Assets<CellTypeDefinition>` store.
fn asset_pairs(
    defs: Vec<CellTypeDefinition>,
) -> Vec<(AssetId<CellTypeDefinition>, CellTypeDefinition)> {
    let mut assets = Assets::<CellTypeDefinition>::default();
    defs.into_iter()
        .map(|def| {
            let handle = assets.add(def.clone());
            (handle.id(), def)
        })
        .collect()
}

// ── Part C behavior 38: store and retrieve by String key ─────────

#[test]
fn registry_stores_and_retrieves_by_string_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("standard", "S", 10.0);
    registry.insert("S".to_owned(), def);
    let retrieved = registry.get("S").expect("registry should contain 'S'");
    assert!(
        (retrieved.hp - 10.0).abs() < f32::EPSILON,
        "retrieved hp should be 10.0, got {}",
        retrieved.hp,
    );
}

#[test]
fn registry_stores_and_retrieves_multi_char_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("guard", "Gu", 15.0);
    registry.insert("Gu".to_owned(), def);
    let retrieved = registry.get("Gu").expect("registry should contain 'Gu'");
    assert!(
        (retrieved.hp - 15.0).abs() < f32::EPSILON,
        "retrieved hp should be 15.0, got {}",
        retrieved.hp,
    );
}

// ── Part C behavior 39: contains() checks String key ─────────────

#[test]
fn registry_contains_string_key() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("S".to_owned(), make_cell_type("standard", "S", 10.0));
    assert!(registry.contains("S"), "'S' should be present");
    assert!(!registry.contains("X"), "'X' should not be present");
    assert!(
        !registry.contains("."),
        "'.' (reserved) should not be present"
    );
}

// ── Part C behavior 40: seed() populates from String-aliased defs ─

#[test]
fn seed_populates_registry_from_cell_type_definitions() {
    let pairs = asset_pairs(vec![
        make_cell_type("standard", "S", 10.0),
        make_cell_type("tough", "T", 30.0),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 2, "registry should contain 2 cell types");
    let standard = registry.get("S").expect("registry should contain 'S'");
    assert!(
        (standard.hp - 10.0).abs() < f32::EPSILON,
        "standard cell hp should be 10.0, got {}",
        standard.hp
    );
}

#[test]
fn seed_populates_with_multi_char_alias() {
    let pairs = asset_pairs(vec![make_cell_type("guard", "Gu", 10.0)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(
        registry.get("Gu").is_some(),
        "multi-char alias 'Gu' should be in registry"
    );
}

// ── Part C behavior 41: seed() panics on dot alias ───────────────

#[test]
#[should_panic(expected = "reserved alias '.'")]
fn seed_panics_on_dot_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", ".", 10.0)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

// ── Part C behavior 42: seed() panics on duplicate String alias ──

#[test]
#[should_panic(expected = "duplicate cell type alias 'S'")]
fn seed_panics_on_duplicate_alias() {
    let pairs = asset_pairs(vec![
        make_cell_type("first", "S", 10.0),
        make_cell_type("second", "S", 20.0),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

// ── Part C behavior 43: seed() skips invalid definitions ─────────

#[test]
fn seed_skips_definitions_with_invalid_hp() {
    let pairs = asset_pairs(vec![
        make_cell_type("valid_a", "A", 10.0),
        make_cell_type("invalid", "B", 0.0), // hp 0.0 fails validate()
        make_cell_type("valid_c", "C", 20.0),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        2,
        "registry should contain 2 valid cell types, skipping the invalid one"
    );
    assert!(
        registry.get("B").is_none(),
        "'B' with hp 0.0 should be skipped"
    );
    assert!(
        registry.get("A").is_some(),
        "'A' should be present after seed()"
    );
    assert!(
        registry.get("C").is_some(),
        "'C' should be present after seed()"
    );
}

// ── Part C behavior 44: update_single() upserts by String alias ──

#[test]
fn update_single_upserts_existing_cell_type_by_alias() {
    let pairs = asset_pairs(vec![make_cell_type("standard", "S", 10.0)]);
    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    // Update 'S' with new hp
    let updated_def = make_cell_type("standard", "S", 20.0);
    let updated_pairs = asset_pairs(vec![updated_def.clone()]);
    let (updated_id, _) = &updated_pairs[0];
    registry.update_single(*updated_id, &updated_def);

    let standard = registry.get("S").expect("'S' should still exist");
    assert!(
        (standard.hp - 20.0).abs() < f32::EPSILON,
        "'S' hp should be updated to 20.0, got {}",
        standard.hp
    );
}

// ── Part C behavior 45: seed() skips definition with empty alias ──

#[test]
fn seed_skips_definition_with_empty_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", "", 10.0)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        0,
        "empty alias should be filtered out by validate() before reaching reserved-alias assert"
    );
}

// ── Existing behavioral tests (updated for String) ───────────────

#[test]
fn seed_clears_existing_entries_before_populating() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", 5.0));
    assert_eq!(registry.len(), 1);

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", 10.0)]);
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        1,
        "registry should contain only 'S' after seed, not 'X'"
    );
    assert!(
        registry.get("X").is_none(),
        "'X' should have been cleared by seed()"
    );
    assert!(
        registry.get("S").is_some(),
        "'S' should be present after seed()"
    );
}

#[test]
fn update_single_ignores_invalid_definition() {
    let pairs = asset_pairs(vec![make_cell_type("standard", "S", 10.0)]);
    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    let invalid_def = make_cell_type("standard", "S", 0.0);
    let invalid_pairs = asset_pairs(vec![invalid_def.clone()]);
    let (invalid_id, _) = &invalid_pairs[0];
    registry.update_single(*invalid_id, &invalid_def);

    let standard = registry.get("S").expect("'S' should still exist");
    assert!(
        (standard.hp - 10.0).abs() < f32::EPSILON,
        "'S' hp should remain 10.0 (invalid update ignored), got {}",
        standard.hp
    );
}

#[test]
fn update_all_resets_and_reseeds_registry() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", 5.0));

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", 10.0)]);
    registry.update_all(&pairs);

    assert_eq!(
        registry.len(),
        1,
        "registry should contain only 'S' after update_all"
    );
    assert!(
        registry.get("X").is_none(),
        "'X' should be gone after update_all"
    );
    assert!(
        registry.get("S").is_some(),
        "'S' should be present after update_all"
    );
}

#[test]
fn asset_dir_returns_cells() {
    assert_eq!(
        CellTypeRegistry::asset_dir(),
        "cells",
        "asset_dir() should return \"cells\""
    );
}

#[test]
fn extensions_returns_cell_ron() {
    assert_eq!(
        CellTypeRegistry::extensions(),
        &["cell.ron"],
        "extensions() should return [\"cell.ron\"]"
    );
}

// ── Part C behaviors 50-52: RON file content after migration ────

#[test]
fn regen_cell_ron_has_regen_behavior() {
    let ron_str = include_str!("../../../assets/cells/regen.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("regen.cell.ron should parse as CellTypeDefinition");
    assert_eq!(
        def.behaviors,
        Some(vec![CellBehavior::Regen { rate: 2.0 }]),
        "regen.cell.ron should have behaviors: Some([Regen {{ rate: 2.0 }}])"
    );
}

#[test]
fn lock_cell_ron_has_no_behaviors() {
    let ron_str = include_str!("../../../assets/cells/lock.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("lock.cell.ron should parse as CellTypeDefinition");
    assert!(
        def.behaviors.is_none(),
        "lock.cell.ron should have behaviors: None, got {:?}",
        def.behaviors
    );
}

#[test]
fn standard_and_tough_cell_rons_have_string_alias_and_no_behaviors() {
    let standard_ron = include_str!("../../../assets/cells/standard.cell.ron");
    let standard: CellTypeDefinition = ron::de::from_str(standard_ron)
        .expect("standard.cell.ron should parse as CellTypeDefinition");
    assert_eq!(
        standard.alias, "S",
        "standard.cell.ron alias should be \"S\""
    );
    assert!(
        standard.behaviors.is_none(),
        "standard.cell.ron should have behaviors: None, got {:?}",
        standard.behaviors
    );

    let tough_ron = include_str!("../../../assets/cells/tough.cell.ron");
    let tough: CellTypeDefinition =
        ron::de::from_str(tough_ron).expect("tough.cell.ron should parse as CellTypeDefinition");
    assert_eq!(tough.alias, "T", "tough.cell.ron alias should be \"T\"");
    assert!(
        tough.behaviors.is_none(),
        "tough.cell.ron should have behaviors: None, got {:?}",
        tough.behaviors
    );
}
