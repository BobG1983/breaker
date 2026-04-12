// ── propagate_registry tests ──────────────────────────────────────

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    registry::{RegistryHandles, SeedableRegistry},
    systems::*,
};

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
struct TestRegistryAsset {
    name:  String,
    value: f32,
}

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

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TestRegistryAsset>()
        .init_resource::<TestRegistry>()
        .add_systems(Update, propagate_registry::<TestRegistry>);
    app
}

/// Helper: creates a test app with a `TestRegistry` pre-seeded with
/// "alpha" -> 1.0 and `RegistryHandles` with loaded=true pointing to
/// that asset.
fn seeded_app() -> (App, Handle<TestRegistryAsset>) {
    let mut app = test_app();

    let h_alpha = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        })
    };

    // Pre-seed the registry.
    app.world_mut()
        .resource_mut::<TestRegistry>()
        .entries
        .insert("alpha".to_string(), 1.0);

    // Insert RegistryHandles with loaded=true.
    let mut rh = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    rh.loaded = true;
    rh.handles = vec![h_alpha.clone()];
    app.insert_resource(rh);

    // Let Added event settle (2 updates).
    app.update();
    app.update();

    (app, h_alpha)
}

/// When no `AssetEvent::Modified` fires, the registry remains
/// unchanged.
#[test]
fn registry_unchanged_when_no_modified_event() {
    let (mut app, _h_alpha) = seeded_app();

    // One more tick with no mutation.
    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert_eq!(
        registry.entries.len(),
        1,
        "TestRegistry should still have 1 entry"
    );
    assert!(
        (registry.entries["alpha"] - 1.0).abs() < f32::EPSILON,
        "alpha should still be 1.0, got {}",
        registry.entries["alpha"]
    );
}

/// When a `Modified` event fires after mutating the asset,
/// `propagate_registry` rebuilds the registry with updated values.
#[test]
fn registry_rebuilt_on_modified_event() {
    let (mut app, h_alpha) = seeded_app();

    // Mutate the asset to trigger Modified.
    {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let asset = assets.get_mut(h_alpha.id()).expect("asset should exist");
        asset.value = 99.0;
    }

    // PostUpdate flushes Modified, First rotates buffer, Update runs
    // system.
    app.update();
    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        (registry.entries["alpha"] - 99.0).abs() < f32::EPSILON,
        "alpha should be 99.0 after Modified event, got {}",
        registry.entries["alpha"]
    );
}

/// When a new asset is added (only `Added` event, no `Modified`),
/// `propagate_registry` does NOT pick up the new asset.
#[test]
fn registry_unchanged_on_added_event_only() {
    let (mut app, _h_alpha) = seeded_app();

    // Add a completely new asset (triggers Added, not Modified).
    {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 50.0,
        });
    }

    // Let Added settle.
    app.update();
    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert_eq!(
        registry.entries.len(),
        1,
        "TestRegistry should still have only 1 entry (alpha)"
    );
    assert!(
        !registry.entries.contains_key("beta"),
        "beta should NOT be in the registry after only an Added event"
    );
}
