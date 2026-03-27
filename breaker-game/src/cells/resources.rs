//! Cells domain resources.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::{GameConfig, prelude::SeedableRegistry};

use super::definition::CellTypeDefinition;

/// Cell configuration resource — shared grid layout properties.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "CellDefaults",
    path = "config/defaults.cells.ron",
    ext = "cells.ron"
)]
pub(crate) struct CellConfig {
    /// Full width of a cell in world units.
    pub width: f32,
    /// Full height of a cell in world units.
    pub height: f32,
    /// Horizontal padding between cells.
    pub padding_x: f32,
    /// Vertical padding between cells.
    pub padding_y: f32,
}

impl Default for CellConfig {
    fn default() -> Self {
        Self {
            width: 70.0,
            height: 24.0,
            padding_x: 4.0,
            padding_y: 4.0,
        }
    }
}

/// Registry mapping alias chars to cell type definitions. Built at boot from all loaded RONs.
#[derive(Resource, Debug, Default)]
pub(crate) struct CellTypeRegistry {
    /// Map from alias char to cell type definition.
    types: HashMap<char, CellTypeDefinition>,
}

impl CellTypeRegistry {
    /// Looks up a cell type definition by its alias char.
    pub(crate) fn get(&self, alias: char) -> Option<&CellTypeDefinition> {
        self.types.get(&alias)
    }

    /// Returns `true` if the registry contains a definition for the given alias.
    pub(crate) fn contains(&self, alias: char) -> bool {
        self.types.contains_key(&alias)
    }

    /// Inserts a cell type definition under the given alias, returning the
    /// previous definition if one existed.
    pub(crate) fn insert(
        &mut self,
        alias: char,
        def: CellTypeDefinition,
    ) -> Option<CellTypeDefinition> {
        self.types.insert(alias, def)
    }

    /// Returns an iterator over all cell type definitions.
    pub(crate) fn values(&self) -> impl Iterator<Item = &CellTypeDefinition> {
        self.types.values()
    }

    /// Returns an iterator over `(alias, definition)` pairs.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&char, &CellTypeDefinition)> {
        self.types.iter()
    }

    /// Returns an iterator over all registered alias chars.
    pub(crate) fn aliases(&self) -> impl Iterator<Item = &char> {
        self.types.keys()
    }

    /// Returns the number of registered cell types.
    pub(crate) fn len(&self) -> usize {
        self.types.len()
    }

    /// Returns `true` if the registry contains no cell types.
    pub(crate) fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Removes all entries from the registry.
    pub(crate) fn clear(&mut self) {
        self.types.clear();
    }
}

impl SeedableRegistry for CellTypeRegistry {
    type Asset = CellTypeDefinition;

    fn asset_dir() -> &'static str {
        "cells"
    }

    fn extensions() -> &'static [&'static str] {
        &["cell.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<CellTypeDefinition>, CellTypeDefinition)]) {
        self.types.clear();
        for (_id, def) in assets {
            if let Err(e) = def.validate() {
                warn!("Skipping cell type '{}': {e}", def.id);
                continue;
            }
            assert!(
                def.alias != '.',
                "cell type '{}' uses reserved alias '.'",
                def.id
            );
            assert!(
                !self.types.contains_key(&def.alias),
                "duplicate cell type alias '{}' from '{}'",
                def.alias,
                def.id
            );
            self.types.insert(def.alias, def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<CellTypeDefinition>, asset: &CellTypeDefinition) {
        if let Err(e) = asset.validate() {
            warn!("Skipping cell type '{}': {e}", asset.id);
            return;
        }
        assert!(
            asset.alias != '.',
            "cell type '{}' uses reserved alias '.'",
            asset.id
        );
        self.types.insert(asset.alias, asset.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::definition::CellBehavior;

    #[test]
    fn cell_defaults_width_height_positive() {
        let config = CellConfig::default();
        assert!(config.width > 0.0);
        assert!(config.height > 0.0);
    }

    #[test]
    fn cell_defaults_ron_parses() {
        let ron_str = include_str!("../../assets/config/defaults.cells.ron");
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
                    def.alias != '.',
                    "{}: alias must not be '.'",
                    path.display()
                );
                assert!(def.hp > 0.0, "{}: hp must be > 0.0", path.display());
            }
        }
    }

    #[test]
    fn registry_detects_duplicate_aliases() {
        let def_a = CellTypeDefinition {
            id: "a".to_owned(),
            alias: 'S',
            hp: 1.0,
            color_rgb: [1.0, 0.0, 0.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
        };
        let def_b = CellTypeDefinition {
            id: "b".to_owned(),
            alias: 'S',
            hp: 2.0,
            color_rgb: [0.0, 1.0, 0.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
        };
        let mut registry = CellTypeRegistry::default();
        registry.insert(def_a.alias, def_a);
        let had_existing = registry.insert(def_b.alias, def_b);
        assert!(
            had_existing.is_some(),
            "inserting duplicate alias should replace"
        );
    }

    // ── SeedableRegistry tests ─────────────────────────────────────

    fn make_cell_type(id: &str, alias: char, hp: f32) -> CellTypeDefinition {
        CellTypeDefinition {
            id: id.to_owned(),
            alias,
            hp,
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 1.0,
            damage_green_min: 0.3,
            damage_blue_range: 0.5,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
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

    // ── Behavior 1: seed() populates from asset pairs ──────────────

    #[test]
    fn seed_populates_registry_from_cell_type_definitions() {
        let pairs = asset_pairs(vec![
            make_cell_type("standard", 'S', 10.0),
            make_cell_type("tough", 'T', 30.0),
        ]);

        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);

        assert_eq!(registry.len(), 2, "registry should contain 2 cell types");
        let standard = registry.get('S').expect("registry should contain 'S'");
        assert!(
            (standard.hp - 10.0).abs() < f32::EPSILON,
            "standard cell hp should be 10.0, got {}",
            standard.hp
        );
    }

    // ── Behavior 2: seed() clears existing entries ─────────────────

    #[test]
    fn seed_clears_existing_entries_before_populating() {
        let mut registry = CellTypeRegistry::default();
        // Pre-populate with 'X'
        registry.insert('X', make_cell_type("old", 'X', 5.0));
        assert_eq!(registry.len(), 1);

        let pairs = asset_pairs(vec![make_cell_type("standard", 'S', 10.0)]);
        registry.seed(&pairs);

        assert_eq!(
            registry.len(),
            1,
            "registry should contain only 'S' after seed, not 'X'"
        );
        assert!(
            registry.get('X').is_none(),
            "'X' should have been cleared by seed()"
        );
        assert!(
            registry.get('S').is_some(),
            "'S' should be present after seed()"
        );
    }

    // ── Behavior 3: seed() skips invalid hp ────────────────────────

    #[test]
    fn seed_skips_definitions_with_invalid_hp() {
        let pairs = asset_pairs(vec![
            make_cell_type("valid_a", 'A', 10.0),
            make_cell_type("invalid", 'B', 0.0), // hp 0.0 fails validate()
            make_cell_type("valid_c", 'C', 20.0),
        ]);

        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);

        assert_eq!(
            registry.len(),
            2,
            "registry should contain 2 valid cell types, skipping the invalid one"
        );
        assert!(
            registry.get('B').is_none(),
            "'B' with hp 0.0 should be skipped"
        );
        assert!(
            registry.get('A').is_some(),
            "'A' should be present after seed()"
        );
        assert!(
            registry.get('C').is_some(),
            "'C' should be present after seed()"
        );
    }

    // ── Behavior 4: seed() panics on dot alias ─────────────────────

    #[test]
    #[should_panic(expected = "reserved alias '.'")]
    fn seed_panics_on_dot_alias() {
        let pairs = asset_pairs(vec![make_cell_type("bad", '.', 10.0)]);

        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);
    }

    // ── Behavior 5: seed() panics on duplicate alias ───────────────

    #[test]
    #[should_panic(expected = "duplicate cell type alias")]
    fn seed_panics_on_duplicate_alias() {
        let pairs = asset_pairs(vec![
            make_cell_type("first", 'S', 10.0),
            make_cell_type("second", 'S', 20.0),
        ]);

        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);
    }

    // ── Behavior 6: update_single() upserts by alias ───────────────

    #[test]
    fn update_single_upserts_existing_cell_type_by_alias() {
        let pairs = asset_pairs(vec![make_cell_type("standard", 'S', 10.0)]);
        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);

        // Update 'S' with new hp
        let updated_def = make_cell_type("standard", 'S', 20.0);
        let updated_pairs = asset_pairs(vec![updated_def.clone()]);
        let (updated_id, _) = &updated_pairs[0];
        registry.update_single(*updated_id, &updated_def);

        let standard = registry.get('S').expect("'S' should still exist");
        assert!(
            (standard.hp - 20.0).abs() < f32::EPSILON,
            "'S' hp should be updated to 20.0, got {}",
            standard.hp
        );
    }

    // ── Behavior 7: update_single() validates ──────────────────────

    #[test]
    fn update_single_ignores_invalid_definition() {
        let pairs = asset_pairs(vec![make_cell_type("standard", 'S', 10.0)]);
        let mut registry = CellTypeRegistry::default();
        registry.seed(&pairs);

        // Attempt to update with invalid hp 0.0
        let invalid_def = make_cell_type("standard", 'S', 0.0);
        let invalid_pairs = asset_pairs(vec![invalid_def.clone()]);
        let (invalid_id, _) = &invalid_pairs[0];
        registry.update_single(*invalid_id, &invalid_def);

        let standard = registry.get('S').expect("'S' should still exist");
        assert!(
            (standard.hp - 10.0).abs() < f32::EPSILON,
            "'S' hp should remain 10.0 (invalid update ignored), got {}",
            standard.hp
        );
    }

    // ── Behavior 8: update_all() resets and re-seeds ───────────────

    #[test]
    fn update_all_resets_and_reseeds_registry() {
        let mut registry = CellTypeRegistry::default();
        // Pre-populate with 'X'
        registry.insert('X', make_cell_type("old", 'X', 5.0));

        let pairs = asset_pairs(vec![make_cell_type("standard", 'S', 10.0)]);
        registry.update_all(&pairs);

        assert_eq!(
            registry.len(),
            1,
            "registry should contain only 'S' after update_all"
        );
        assert!(
            registry.get('X').is_none(),
            "'X' should be gone after update_all"
        );
        assert!(
            registry.get('S').is_some(),
            "'S' should be present after update_all"
        );
    }

    // ── Behavior 9: asset_dir() and extensions() ───────────────────

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
}
