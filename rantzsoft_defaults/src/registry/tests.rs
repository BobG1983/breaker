//! Tests for `SeedableRegistry` trait and `RegistryHandles` resource.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use super::*;

/// Test asset type for registry tests.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
struct TestRegistryAsset {
    name:  String,
    value: f32,
}

/// Test registry type implementing `SeedableRegistry`.
#[derive(Resource, Default, Debug)]
struct TestRegistry {
    entries: HashMap<String, f32>,
}

impl SeedableRegistry for TestRegistry {
    type Asset = TestRegistryAsset;

    fn asset_dir() -> &'static str {
        "test_registry"
    }

    fn extensions() -> &'static [&'static str] {
        &["test.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<TestRegistryAsset>, TestRegistryAsset)]) {
        self.entries.clear();
        for (_id, asset) in assets {
            self.entries.insert(asset.name.clone(), asset.value);
        }
    }

    fn update_single(&mut self, _id: AssetId<TestRegistryAsset>, asset: &TestRegistryAsset) {
        self.entries.insert(asset.name.clone(), asset.value);
    }
}

// ── Behavior 1: seed() populates registry from assets ─────────────

/// `seed()` populates the registry from the given asset pairs.
#[test]
fn seed_populates_registry_from_assets() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<TestRegistryAsset>();

    let (id_alpha, id_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha.id(), h_beta.id())
    };

    let mut registry = TestRegistry::default();
    let pairs = vec![
        (
            id_alpha,
            TestRegistryAsset {
                name:  "alpha".to_string(),
                value: 1.0,
            },
        ),
        (
            id_beta,
            TestRegistryAsset {
                name:  "beta".to_string(),
                value: 2.0,
            },
        ),
    ];
    registry.seed(&pairs);

    assert_eq!(registry.entries.len(), 2);
    assert!(
        (registry.entries["alpha"] - 1.0).abs() < f32::EPSILON,
        "alpha should be 1.0, got {}",
        registry.entries["alpha"]
    );
    assert!(
        (registry.entries["beta"] - 2.0).abs() < f32::EPSILON,
        "beta should be 2.0, got {}",
        registry.entries["beta"]
    );
}

/// `seed()` on a non-empty registry replaces all existing entries.
#[test]
fn seed_replaces_existing_entries() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<TestRegistryAsset>();

    let (id_alpha, id_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha.id(), h_beta.id())
    };

    let mut registry = TestRegistry::default();
    // Pre-populate with "gamma"
    registry.entries.insert("gamma".to_string(), 99.0);

    let pairs = vec![
        (
            id_alpha,
            TestRegistryAsset {
                name:  "alpha".to_string(),
                value: 1.0,
            },
        ),
        (
            id_beta,
            TestRegistryAsset {
                name:  "beta".to_string(),
                value: 2.0,
            },
        ),
    ];
    registry.seed(&pairs);

    assert_eq!(
        registry.entries.len(),
        2,
        "only alpha and beta should remain, gamma should be gone"
    );
    assert!(registry.entries.contains_key("alpha"));
    assert!(registry.entries.contains_key("beta"));
    assert!(
        !registry.entries.contains_key("gamma"),
        "gamma should have been replaced"
    );
}

// ── Behavior 2: update_all resets and re-seeds ────────────────────

/// `update_all()` resets registry to default then seeds with new assets.
#[test]
fn update_all_resets_and_reseeds() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<TestRegistryAsset>();

    let (id_new_a, id_new_b) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_a = assets.add(TestRegistryAsset {
            name:  "new_a".to_string(),
            value: 1.0,
        });
        let h_b = assets.add(TestRegistryAsset {
            name:  "new_b".to_string(),
            value: 2.0,
        });
        (h_a.id(), h_b.id())
    };

    let mut registry = TestRegistry::default();
    registry.entries.insert("old".to_string(), 0.0);

    let pairs = vec![
        (
            id_new_a,
            TestRegistryAsset {
                name:  "new_a".to_string(),
                value: 1.0,
            },
        ),
        (
            id_new_b,
            TestRegistryAsset {
                name:  "new_b".to_string(),
                value: 2.0,
            },
        ),
    ];
    registry.update_all(&pairs);

    assert_eq!(registry.entries.len(), 2);
    assert!(registry.entries.contains_key("new_a"));
    assert!(registry.entries.contains_key("new_b"));
    assert!(
        !registry.entries.contains_key("old"),
        "old should be gone after update_all"
    );
}

/// `update_all()` with an empty slice produces an empty registry.
#[test]
fn update_all_with_empty_slice_produces_empty_registry() {
    let mut registry = TestRegistry::default();
    registry.entries.insert("old".to_string(), 0.0);

    registry.update_all(&[]);

    assert!(
        registry.entries.is_empty(),
        "update_all with empty slice should produce empty registry"
    );
}

// ── Behavior 3: update_single upserts by name ────────────────────

/// `update_single()` updates an existing entry.
#[test]
fn update_single_updates_existing_entry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<TestRegistryAsset>();

    let (id_alpha, id_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha.id(), h_beta.id())
    };

    let mut registry = TestRegistry::default();
    let pairs = vec![
        (
            id_alpha,
            TestRegistryAsset {
                name:  "alpha".to_string(),
                value: 1.0,
            },
        ),
        (
            id_beta,
            TestRegistryAsset {
                name:  "beta".to_string(),
                value: 2.0,
            },
        ),
    ];
    registry.seed(&pairs);

    // Update alpha to 99.0
    registry.update_single(
        id_alpha,
        &TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 99.0,
        },
    );

    assert!(
        (registry.entries["alpha"] - 99.0).abs() < f32::EPSILON,
        "alpha should be 99.0 after update_single, got {}",
        registry.entries["alpha"]
    );
    assert!(
        (registry.entries["beta"] - 2.0).abs() < f32::EPSILON,
        "beta should be unchanged at 2.0, got {}",
        registry.entries["beta"]
    );
}

/// `update_single()` with a new name inserts it.
#[test]
fn update_single_inserts_new_entry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<TestRegistryAsset>();

    let id_gamma = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        assets
            .add(TestRegistryAsset {
                name:  "gamma".to_string(),
                value: 3.0,
            })
            .id()
    };

    let mut registry = TestRegistry::default();
    registry.entries.insert("alpha".to_string(), 1.0);

    registry.update_single(
        id_gamma,
        &TestRegistryAsset {
            name:  "gamma".to_string(),
            value: 3.0,
        },
    );

    assert_eq!(registry.entries.len(), 2, "should have alpha and gamma");
    assert!(
        (registry.entries["gamma"] - 3.0).abs() < f32::EPSILON,
        "gamma should be 3.0, got {}",
        registry.entries["gamma"]
    );
}

// ── Behavior 4: RegistryHandles is a Resource ────────────────────

/// `RegistryHandles` starts unloaded with empty handles.
#[test]
fn registry_handles_starts_unloaded_with_empty_handles() {
    let handles = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    assert!(!handles.loaded, "loaded should be false on creation");
    assert!(
        handles.handles.is_empty(),
        "handles should be empty on creation"
    );
}

/// `RegistryHandles` can be inserted into an `App` as a `Resource`.
#[test]
fn registry_handles_is_a_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let handles = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    app.insert_resource(handles);
    assert!(
        app.world()
            .get_resource::<RegistryHandles<TestRegistryAsset>>()
            .is_some(),
        "RegistryHandles should be insertable as a Resource"
    );
}
