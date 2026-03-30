// ── init_defaults_handle tests ───────────────────────────────────

use bevy::prelude::*;

use super::helpers::TestDefaults;
use crate::{handle::DefaultsHandle, systems::*};

/// After the startup system runs, `DefaultsHandle<TestDefaults>`
/// should exist as a resource.
#[test]
fn inserts_defaults_handle_on_startup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TestDefaults>()
        .add_systems(Startup, init_defaults_handle::<TestDefaults>);

    app.update();

    assert!(
        app.world()
            .get_resource::<DefaultsHandle<TestDefaults>>()
            .is_some(),
        "DefaultsHandle<TestDefaults> should exist after startup"
    );
}

// ── init_registry_handles tests ───────────────────────────────────

use std::collections::HashMap;

use serde::Deserialize;

use crate::registry::{RegistryHandles, SeedableRegistry};

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
struct TestRegistryAsset {
    name: String,
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

/// After `init_registry_handles` runs at startup,
/// `RegistryHandles<TestRegistryAsset>` should exist with
/// `loaded == false` and empty handles.
#[test]
fn inserts_registry_handles_on_startup() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TestRegistryAsset>()
        .init_asset::<bevy::asset::LoadedFolder>()
        .add_systems(Startup, init_registry_handles::<TestRegistry>);

    app.update();

    let handles = app
        .world()
        .get_resource::<RegistryHandles<TestRegistryAsset>>()
        .expect("RegistryHandles<TestRegistryAsset> should exist after startup");
    assert!(!handles.loaded, "loaded should be false after init");
    assert!(
        handles.handles.is_empty(),
        "handles should be empty after init"
    );
}
