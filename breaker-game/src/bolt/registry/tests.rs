use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::data::*;
use crate::bolt::definition::BoltDefinition;

/// Creates a `BoltDefinition` for testing with the given name and
/// `base_damage`.
fn make_bolt_definition(name: &str, base_damage: f32) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Helper: creates an `App` with `AssetPlugin` and returns `AssetId`s for
/// a list of `BoltDefinition` values. This gives us real (non-default)
/// `AssetId`s backed by the Bevy asset system.
fn asset_ids_for(defs: &[BoltDefinition]) -> (App, Vec<(AssetId<BoltDefinition>, BoltDefinition)>) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<BoltDefinition>();

    let pairs: Vec<_> = {
        let mut assets = app.world_mut().resource_mut::<Assets<BoltDefinition>>();
        defs.iter()
            .map(|d| {
                let handle = assets.add(d.clone());
                (handle.id(), d.clone())
            })
            .collect()
    };

    (app, pairs)
}

// ── Behavior 7: Default registry is empty ────────────────────

#[test]
fn default_registry_is_empty() {
    let registry = BoltRegistry::default();
    assert!(registry.is_empty(), "default BoltRegistry should be empty");
    assert_eq!(registry.len(), 0, "default BoltRegistry len() should be 0");
}

// ── Behavior 8: Insert and lookup by name ────────────────────

#[test]
fn insert_and_lookup_by_name() {
    let mut registry = BoltRegistry::default();
    let def = make_bolt_definition("Bolt", 10.0);
    registry.insert("Bolt".to_string(), def);
    assert!(
        registry.get("Bolt").is_some(),
        "get('Bolt') should return Some after insert"
    );
    assert!(
        registry.contains("Bolt"),
        "contains('Bolt') should return true after insert"
    );
    assert!(
        (registry.get("Bolt").unwrap().base_speed - 720.0).abs() < f32::EPSILON,
        "inserted bolt should have base_speed 720.0"
    );
}

#[test]
fn lookup_nonexistent_returns_none() {
    let registry = BoltRegistry::default();
    assert!(
        registry.get("Nonexistent").is_none(),
        "get('Nonexistent') on empty registry should return None"
    );
    assert!(
        !registry.contains("Nonexistent"),
        "contains('Nonexistent') on empty registry should return false"
    );
}

// ── Behavior 9: seed() populates from asset pairs ────────────

#[test]
fn seed_populates_registry_from_bolt_definitions() {
    let bolt = make_bolt_definition("Bolt", 10.0);
    let heavy = make_bolt_definition("Heavy", 25.0);
    let (_app, pairs) = asset_ids_for(&[bolt, heavy]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 2, "registry should contain 2 bolts");
    assert!(
        (registry.get("Bolt").unwrap().base_damage - 10.0).abs() < f32::EPSILON,
        "Bolt base_damage should be 10.0"
    );
    assert!(
        (registry.get("Heavy").unwrap().base_damage - 25.0).abs() < f32::EPSILON,
        "Heavy base_damage should be 25.0"
    );
}

#[test]
fn seed_names_iterator_yields_all_entries() {
    let bolt = make_bolt_definition("Bolt", 10.0);
    let heavy = make_bolt_definition("Heavy", 25.0);
    let (_app, pairs) = asset_ids_for(&[bolt, heavy]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs);

    let mut names: Vec<_> = registry.names().cloned().collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Bolt".to_string(), "Heavy".to_string()],
        "names() should yield both 'Bolt' and 'Heavy'"
    );
}

// ── Behavior 10: seed() clears existing entries before populating ──

#[test]
fn seed_clears_existing_entries() {
    let old = make_bolt_definition("Old", 5.0);
    let bolt = make_bolt_definition("Bolt", 10.0);
    let (_app, pairs_old) = asset_ids_for(&[old]);
    let (_app2, pairs_new) = asset_ids_for(&[bolt]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs_old);
    assert_eq!(registry.len(), 1);
    assert!(registry.get("Old").is_some());

    registry.seed(&pairs_new);

    assert_eq!(registry.len(), 1, "after re-seed, only Bolt should remain");
    assert!(
        registry.get("Bolt").is_some(),
        "Bolt should be present after re-seed"
    );
    assert!(
        registry.get("Old").is_none(),
        "Old should be gone after re-seed"
    );
}

#[test]
fn seed_triple_reseed_only_last_entries_remain() {
    let a = make_bolt_definition("A", 1.0);
    let b = make_bolt_definition("B", 2.0);
    let c = make_bolt_definition("C", 3.0);
    let (_app_a, pairs_a) = asset_ids_for(&[a]);
    let (_app_b, pairs_b) = asset_ids_for(&[b]);
    let (_app_c, pairs_c) = asset_ids_for(&[c]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs_a);
    registry.seed(&pairs_b);
    registry.seed(&pairs_c);

    assert_eq!(registry.len(), 1, "only C should remain");
    assert!(registry.get("C").is_some());
    assert!(registry.get("A").is_none());
    assert!(registry.get("B").is_none());
}

// ── Behavior 11: seed() skips duplicate bolt name with warning ──

#[test]
fn seed_skips_duplicate_bolt_name() {
    let bolt1 = make_bolt_definition("Bolt", 10.0);
    let bolt2 = make_bolt_definition("Bolt", 25.0);
    let (_app, pairs) = asset_ids_for(&[bolt1, bolt2]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1, "duplicate should be skipped");
    assert!(
        (registry.get("Bolt").unwrap().base_damage - 10.0).abs() < f32::EPSILON,
        "first occurrence (base_damage=10.0) should win"
    );
}

// ── Behavior 12: update_single() upserts existing entry by name ──

#[test]
fn update_single_upserts_existing_entry_by_name() {
    let bolt = make_bolt_definition("Bolt", 10.0);
    let (_app, pairs) = asset_ids_for(&[bolt]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs);
    assert!(
        (registry.get("Bolt").unwrap().base_damage - 10.0).abs() < f32::EPSILON,
        "Bolt base_damage should be 10.0 initially"
    );

    let updated = make_bolt_definition("Bolt", 15.0);
    let id = pairs[0].0;
    registry.update_single(id, &updated);

    assert!(
        (registry.get("Bolt").unwrap().base_damage - 15.0).abs() < f32::EPSILON,
        "Bolt base_damage should be 15.0 after update_single"
    );
    assert_eq!(registry.len(), 1, "registry len should still be 1");
}

#[test]
fn update_single_inserts_new_entry_when_name_absent() {
    let bolt = make_bolt_definition("Bolt", 10.0);
    let (_app, pairs) = asset_ids_for(&[bolt]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs);

    let heavy = make_bolt_definition("Heavy", 25.0);
    let id = pairs[0].0; // reuse id -- update_single upserts by name
    registry.update_single(id, &heavy);

    assert!(
        registry.get("Heavy").is_some(),
        "Heavy should be present after update_single with new name"
    );
}

// ── Behavior 13: update_all() resets and re-seeds ────────────

#[test]
fn update_all_resets_and_reseeds() {
    let old = make_bolt_definition("Old", 5.0);
    let bolt = make_bolt_definition("Bolt", 10.0);
    let (_app, pairs_old) = asset_ids_for(&[old]);
    let (_app2, pairs_new) = asset_ids_for(&[bolt]);

    let mut registry = BoltRegistry::default();
    registry.seed(&pairs_old);
    assert!(registry.get("Old").is_some());

    registry.update_all(&pairs_new);

    assert_eq!(
        registry.len(),
        1,
        "after update_all, only Bolt should remain"
    );
    assert!(
        registry.get("Bolt").is_some(),
        "Bolt should be present after update_all"
    );
    assert!(
        registry.get("Old").is_none(),
        "Old should be gone after update_all"
    );
}

// ── Behavior 14: asset_dir() returns "bolts" ─────────────────

#[test]
fn asset_dir_returns_bolts() {
    assert_eq!(
        BoltRegistry::asset_dir(),
        "bolts",
        "asset_dir() should return \"bolts\""
    );
}

// ── Behavior 15: extensions() returns ["bolt.ron"] ───────────

#[test]
fn extensions_returns_bolt_ron() {
    assert_eq!(
        BoltRegistry::extensions(),
        &["bolt.ron"],
        "extensions() should return [\"bolt.ron\"]"
    );
}

// ── Behavior 41: insert() with a name not yet in the registry adds it ──

#[test]
fn insert_adds_new_entry() {
    let mut registry = BoltRegistry::default();
    registry.insert("Bolt".to_string(), make_bolt_definition("Bolt", 10.0));
    assert_eq!(registry.len(), 1, "len should be 1 after insert");
    assert!(
        (registry.get("Bolt").unwrap().base_damage - 10.0).abs() < f32::EPSILON,
        "inserted bolt base_damage should be 10.0"
    );
}

// ── Behavior 42: insert() with existing name silently overwrites ──

#[test]
fn insert_overwrites_existing_entry_silently() {
    let mut registry = BoltRegistry::default();
    registry.insert("Bolt".to_string(), make_bolt_definition("Bolt", 10.0));
    registry.insert("Bolt".to_string(), make_bolt_definition("Bolt", 25.0));
    assert_eq!(
        registry.len(),
        1,
        "len should still be 1 after overwrite (not 2)"
    );
    assert!(
        (registry.get("Bolt").unwrap().base_damage - 25.0).abs() < f32::EPSILON,
        "overwritten bolt base_damage should be 25.0"
    );
}
