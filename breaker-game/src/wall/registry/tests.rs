use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::{super::definition::WallDefinition, core::*};

/// Creates a `WallDefinition` for testing with the given name and
/// `half_thickness`.
fn make_wall(name: &str, half_thickness: f32) -> WallDefinition {
    WallDefinition {
        name: name.to_owned(),
        half_thickness,
        ..WallDefinition::default()
    }
}

/// Helper: creates an `App` with `AssetPlugin` and returns `AssetId`s for
/// a list of `WallDefinition` values. This gives us real (non-default)
/// `AssetId`s backed by the Bevy asset system.
fn asset_ids_for(defs: &[WallDefinition]) -> (App, Vec<(AssetId<WallDefinition>, WallDefinition)>) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<WallDefinition>();

    let pairs: Vec<_> = {
        let mut assets = app.world_mut().resource_mut::<Assets<WallDefinition>>();
        defs.iter()
            .map(|d| {
                let handle = assets.add(d.clone());
                (handle.id(), d.clone())
            })
            .collect()
    };

    (app, pairs)
}

// ── Behavior 13: Default WallRegistry is empty ──────────────────

#[test]
fn default_registry_is_empty() {
    let registry = WallRegistry::default();
    assert!(registry.is_empty(), "default WallRegistry should be empty");
    assert_eq!(registry.len(), 0, "default WallRegistry len() should be 0");
}

#[test]
fn default_registry_get_returns_none() {
    let registry = WallRegistry::default();
    assert!(
        registry.get("anything").is_none(),
        "get on empty registry should return None"
    );
}

// ── Behavior 14: Insert and lookup by name ──────────────────────

#[test]
fn insert_and_lookup_by_name() {
    let mut registry = WallRegistry::default();
    let def = make_wall("Wall", 90.0);
    registry.insert("Wall".to_string(), def);
    assert!(
        registry.get("Wall").is_some(),
        "get('Wall') should return Some after insert"
    );
    assert_eq!(registry.get("Wall").unwrap().name, "Wall");
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 90.0).abs() < f32::EPSILON,
        "inserted wall should have half_thickness 90.0"
    );
}

#[test]
fn lookup_nonexistent_returns_none() {
    let mut registry = WallRegistry::default();
    let def = make_wall("Wall", 90.0);
    registry.insert("Wall".to_string(), def);
    assert!(
        registry.get("Nonexistent").is_none(),
        "get('Nonexistent') should return None"
    );
    assert!(
        registry.contains("Wall"),
        "contains('Wall') should return true"
    );
    assert!(
        !registry.contains("Nonexistent"),
        "contains('Nonexistent') should return false"
    );
}

// ── Behavior 15: seed() populates registry from asset pairs ─────

#[test]
fn seed_populates_registry_from_wall_definitions() {
    let wall = make_wall("Wall", 90.0);
    let thick = make_wall("ThickWall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall, thick]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 2, "registry should contain 2 walls");
    assert!(
        registry.get("Wall").is_some(),
        "registry should contain Wall"
    );
    assert!(
        registry.get("ThickWall").is_some(),
        "registry should contain ThickWall"
    );
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 90.0).abs() < f32::EPSILON,
        "Wall half_thickness should be 90.0"
    );
    assert!(
        (registry.get("ThickWall").unwrap().half_thickness - 150.0).abs() < f32::EPSILON,
        "ThickWall half_thickness should be 150.0"
    );
}

#[test]
fn seed_names_iterator_yields_all_entries() {
    let wall = make_wall("Wall", 90.0);
    let thick = make_wall("ThickWall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall, thick]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    let mut names: Vec<_> = registry.names().cloned().collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec!["ThickWall".to_string(), "Wall".to_string()],
        "names() should yield both 'ThickWall' and 'Wall'"
    );
}

// ── Behavior 16: seed() clears existing entries before populating ──

#[test]
fn seed_clears_existing_entries() {
    let old = make_wall("Old", 60.0);
    let wall = make_wall("Wall", 90.0);
    let (_app, pairs_old) = asset_ids_for(&[old]);
    let (_app2, pairs_new) = asset_ids_for(&[wall]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs_old);
    assert_eq!(registry.len(), 1);
    assert!(registry.get("Old").is_some());

    registry.seed(&pairs_new);

    assert_eq!(registry.len(), 1, "after re-seed, only Wall should remain");
    assert!(
        registry.get("Wall").is_some(),
        "Wall should be present after re-seed"
    );
    assert!(
        registry.get("Old").is_none(),
        "Old should be gone after re-seed"
    );
}

#[test]
fn seed_triple_reseed_only_last_entries_remain() {
    let a = make_wall("A", 10.0);
    let b = make_wall("B", 20.0);
    let c = make_wall("C", 30.0);
    let (_app_a, pairs_a) = asset_ids_for(&[a]);
    let (_app_b, pairs_b) = asset_ids_for(&[b]);
    let (_app_c, pairs_c) = asset_ids_for(&[c]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs_a);
    registry.seed(&pairs_b);
    registry.seed(&pairs_c);

    assert_eq!(registry.len(), 1, "only C should remain");
    assert!(registry.get("C").is_some());
    assert!(registry.get("A").is_none());
    assert!(registry.get("B").is_none());
}

// ── Behavior 17: seed() skips duplicate wall name with warning ──

#[test]
fn seed_skips_duplicate_wall_name() {
    let wall1 = make_wall("Wall", 90.0);
    let wall2 = make_wall("Wall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall1, wall2]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1, "duplicate should be skipped");
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 90.0).abs() < f32::EPSILON,
        "first occurrence (half_thickness=90.0) should win"
    );
}

#[test]
fn seed_skips_triple_duplicate_keeps_first() {
    let wall1 = make_wall("Wall", 90.0);
    let wall2 = make_wall("Wall", 150.0);
    let wall3 = make_wall("Wall", 200.0);
    let (_app, pairs) = asset_ids_for(&[wall1, wall2, wall3]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        1,
        "three duplicates should result in 1 entry"
    );
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 90.0).abs() < f32::EPSILON,
        "first occurrence (half_thickness=90.0) should win"
    );
}

// ── Behavior 18: update_single() upserts existing entry by name ──

#[test]
fn update_single_upserts_existing_entry_by_name() {
    let wall = make_wall("Wall", 90.0);
    let (_app, pairs) = asset_ids_for(&[wall]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 90.0).abs() < f32::EPSILON,
        "Wall half_thickness should be 90.0 initially"
    );

    let updated = make_wall("Wall", 50.0);
    let id = pairs[0].0;
    registry.update_single(id, &updated);

    assert!(
        (registry.get("Wall").unwrap().half_thickness - 50.0).abs() < f32::EPSILON,
        "Wall half_thickness should be 50.0 after update_single"
    );
    assert_eq!(registry.len(), 1, "registry len should still be 1");
}

#[test]
fn update_single_inserts_new_entry_when_name_absent() {
    let wall = make_wall("Wall", 90.0);
    let (_app, pairs) = asset_ids_for(&[wall]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    let thick = make_wall("ThickWall", 150.0);
    let id = pairs[0].0; // reuse id — update_single upserts by name
    registry.update_single(id, &thick);

    assert!(
        registry.get("ThickWall").is_some(),
        "ThickWall should be present after update_single with new name"
    );
    assert_eq!(
        registry.len(),
        2,
        "registry should have 2 entries after inserting new name"
    );
}

// ── Behavior 19: update_all() resets and re-seeds ───────────────

#[test]
fn update_all_resets_and_reseeds() {
    let old = make_wall("Old", 60.0);
    let wall = make_wall("Wall", 90.0);
    let (_app, pairs_old) = asset_ids_for(&[old]);
    let (_app2, pairs_new) = asset_ids_for(&[wall]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs_old);
    assert!(registry.get("Old").is_some());

    registry.update_all(&pairs_new);

    assert_eq!(
        registry.len(),
        1,
        "after update_all, only Wall should remain"
    );
    assert!(
        registry.get("Wall").is_some(),
        "Wall should be present after update_all"
    );
    assert!(
        registry.get("Old").is_none(),
        "Old should be gone after update_all"
    );
}

#[test]
fn update_all_with_empty_pairs_clears_registry() {
    let wall = make_wall("Wall", 90.0);
    let (_app, pairs) = asset_ids_for(&[wall]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);
    assert_eq!(registry.len(), 1);

    let empty_pairs: Vec<(AssetId<WallDefinition>, WallDefinition)> = vec![];
    registry.update_all(&empty_pairs);

    assert!(
        registry.is_empty(),
        "registry should be empty after update_all with empty pairs"
    );
}

// ── Behavior 20: asset_dir() returns "walls" ────────────────────

#[test]
fn asset_dir_returns_walls() {
    assert_eq!(
        WallRegistry::asset_dir(),
        "walls",
        "asset_dir() should return \"walls\""
    );
}

// ── Behavior 21: extensions() returns ["wall.ron"] ──────────────

#[test]
fn extensions_returns_wall_ron() {
    assert_eq!(
        WallRegistry::extensions(),
        &["wall.ron"],
        "extensions() should return [\"wall.ron\"]"
    );
}

// ── Behavior 22: insert() adds new entry to empty registry ──────

#[test]
fn insert_adds_new_entry_to_empty_registry() {
    let mut registry = WallRegistry::default();
    registry.insert("Wall".to_string(), make_wall("Wall", 90.0));
    assert!(registry.get("Wall").is_some(), "Wall should be present");
    assert_eq!(registry.len(), 1, "len should be 1 after insert");
}

#[test]
fn insert_second_distinct_name_increases_len() {
    let mut registry = WallRegistry::default();
    registry.insert("Wall".to_string(), make_wall("Wall", 90.0));
    registry.insert("ThickWall".to_string(), make_wall("ThickWall", 150.0));
    assert_eq!(
        registry.len(),
        2,
        "len should be 2 after two distinct inserts"
    );
}

// ── Behavior 23: insert() with existing name silently overwrites ──

#[test]
fn insert_overwrites_existing_entry_silently() {
    let mut registry = WallRegistry::default();
    registry.insert("Wall".to_string(), make_wall("Wall", 90.0));
    registry.insert("Wall".to_string(), make_wall("Wall", 50.0));
    assert_eq!(
        registry.len(),
        1,
        "len should still be 1 after overwrite (not 2)"
    );
    assert!(
        (registry.get("Wall").unwrap().half_thickness - 50.0).abs() < f32::EPSILON,
        "overwritten wall half_thickness should be 50.0"
    );
}

// ── Behavior 24: clear() removes all entries ────────────────────

#[test]
fn clear_removes_all_entries() {
    let wall = make_wall("Wall", 90.0);
    let thick = make_wall("ThickWall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall, thick]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);
    assert_eq!(registry.len(), 2);

    registry.clear();

    assert!(registry.is_empty(), "registry should be empty after clear");
    assert_eq!(registry.len(), 0, "len should be 0 after clear");
}

#[test]
fn clear_on_empty_registry_is_noop() {
    let mut registry = WallRegistry::default();
    registry.clear();
    assert!(
        registry.is_empty(),
        "clear on empty registry should remain empty"
    );
}

// ── Behavior 25: iter() yields all (name, definition) pairs ─────

#[test]
fn iter_yields_all_pairs() {
    let wall = make_wall("Wall", 90.0);
    let thick = make_wall("ThickWall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall, thick]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    let collected: Vec<_> = registry.iter().collect();
    assert_eq!(collected.len(), 2, "iter should yield 2 pairs");

    let mut names: Vec<_> = collected.iter().map(|(name, _)| name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec!["ThickWall", "Wall"],
        "iter names (sorted) should be ['ThickWall', 'Wall']"
    );
}

#[test]
fn values_count_matches_len() {
    let wall = make_wall("Wall", 90.0);
    let thick = make_wall("ThickWall", 150.0);
    let (_app, pairs) = asset_ids_for(&[wall, thick]);

    let mut registry = WallRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.values().count(),
        2,
        "values() count should match len()"
    );
}
