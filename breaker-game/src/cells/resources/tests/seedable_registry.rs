use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::super::data::*;
use crate::cells::definition::{CellTypeDefinition, Toughness};

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

// ── SeedableRegistry tests ─────────────────────────────────────

#[test]
fn registry_stores_and_retrieves_by_string_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("standard", "S", Toughness::Standard);
    registry.insert("S".to_owned(), def);
    let retrieved = registry.get("S").expect("registry should contain 'S'");
    assert_eq!(
        retrieved.toughness,
        Toughness::Standard,
        "retrieved toughness should be Standard"
    );
}

#[test]
fn registry_stores_and_retrieves_multi_char_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("guard", "Gu", Toughness::Weak);
    registry.insert("Gu".to_owned(), def);
    let retrieved = registry.get("Gu").expect("registry should contain 'Gu'");
    assert_eq!(retrieved.toughness, Toughness::Weak);
}

#[test]
fn registry_contains_string_key() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "S".to_owned(),
        make_cell_type("standard", "S", Toughness::Standard),
    );
    assert!(registry.contains("S"), "'S' should be present");
    assert!(!registry.contains("X"), "'X' should not be present");
    assert!(
        !registry.contains("."),
        "'.' (reserved) should not be present"
    );
}

#[test]
fn seed_populates_registry_from_cell_type_definitions() {
    let pairs = asset_pairs(vec![
        make_cell_type("standard", "S", Toughness::Standard),
        make_cell_type("tough", "T", Toughness::Tough),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 2, "registry should contain 2 cell types");
    let standard = registry.get("S").expect("registry should contain 'S'");
    assert_eq!(standard.toughness, Toughness::Standard);
}

#[test]
fn seed_populates_with_multi_char_alias() {
    let pairs = asset_pairs(vec![make_cell_type("guard", "Gu", Toughness::Weak)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(
        registry.get("Gu").is_some(),
        "multi-char alias 'Gu' should be in registry"
    );
}

#[test]
#[should_panic(expected = "reserved alias '.'")]
fn seed_panics_on_dot_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", ".", Toughness::Standard)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

#[test]
#[should_panic(expected = "duplicate cell type alias 'S'")]
fn seed_panics_on_duplicate_alias() {
    let pairs = asset_pairs(vec![
        make_cell_type("first", "S", Toughness::Standard),
        make_cell_type("second", "S", Toughness::Tough),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

#[test]
fn seed_skips_definition_with_empty_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", "", Toughness::Standard)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        0,
        "empty alias should be filtered out by validate()"
    );
}

#[test]
fn seed_clears_existing_entries_before_populating() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", Toughness::Weak));
    assert_eq!(registry.len(), 1);

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(registry.get("X").is_none());
    assert!(registry.get("S").is_some());
}

#[test]
fn update_single_upserts_existing_cell_type_by_alias() {
    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    let updated_def = make_cell_type("standard", "S", Toughness::Tough);
    let updated_pairs = asset_pairs(vec![updated_def.clone()]);
    let (updated_id, _) = &updated_pairs[0];
    registry.update_single(*updated_id, &updated_def);

    let standard = registry.get("S").expect("'S' should still exist");
    assert_eq!(
        standard.toughness,
        Toughness::Tough,
        "toughness should be updated to Tough"
    );
}

#[test]
fn update_all_resets_and_reseeds_registry() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", Toughness::Weak));

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    registry.update_all(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(registry.get("X").is_none());
    assert!(registry.get("S").is_some());
}

#[test]
fn asset_dir_returns_cells() {
    assert_eq!(CellTypeRegistry::asset_dir(), "cells");
}

#[test]
fn extensions_returns_cell_ron() {
    assert_eq!(CellTypeRegistry::extensions(), &["cell.ron"]);
}
