// ── seed_registry tests ───────────────────────────────────────────

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
        .init_asset::<bevy::asset::LoadedFolder>()
        .init_resource::<TestRegistry>()
        .add_systems(Update, seed_registry::<TestRegistry>.map(drop));
    app
}

/// Without `RegistryHandles`, `seed_registry` reports zero progress
/// and the `TestRegistry` stays empty.
#[test]
fn returns_zero_progress_when_handles_missing() {
    let mut app = test_app();

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        registry.entries.is_empty(),
        "TestRegistry should remain empty when RegistryHandles is absent"
    );
}

/// When `RegistryHandles` exists but folder is not in
/// `Assets<LoadedFolder>`, `seed_registry` reports zero progress.
#[test]
fn returns_zero_progress_when_folder_not_loaded() {
    let mut app = test_app();

    // Insert handles with a default folder handle (pointing to
    // nothing).
    let handles = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    app.insert_resource(handles);

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        registry.entries.is_empty(),
        "TestRegistry should remain empty when folder is not loaded"
    );
    let rh = app.world().resource::<RegistryHandles<TestRegistryAsset>>();
    assert!(
        !rh.loaded,
        "loaded should remain false when folder not available"
    );
}

/// When `LoadedFolder` is available with matching assets,
/// `seed_registry` resolves the folder, extracts handles, and seeds
/// the registry.
#[test]
fn seeds_registry_when_loaded_folder_available() {
    let mut app = test_app();

    // Create typed assets.
    let (h_alpha, h_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha, h_beta)
    };

    // Create a LoadedFolder containing those handles as untyped.
    let folder_handle = {
        let mut loaded_folders = app
            .world_mut()
            .resource_mut::<Assets<bevy::asset::LoadedFolder>>();
        loaded_folders.add(bevy::asset::LoadedFolder {
            handles: vec![h_alpha.into(), h_beta.into()],
        })
    };

    // Insert RegistryHandles pointing to the loaded folder.
    let mut rh = RegistryHandles::<TestRegistryAsset>::new(folder_handle);
    rh.loaded = false;
    app.insert_resource(rh);

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert_eq!(
        registry.entries.len(),
        2,
        "TestRegistry should contain 2 entries"
    );
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

    let rh = app.world().resource::<RegistryHandles<TestRegistryAsset>>();
    assert!(rh.loaded, "loaded should be true after seeding");
    assert_eq!(rh.handles.len(), 2, "handles should contain 2 entries");
}

/// When `RegistryHandles` has `loaded=true` and handles already
/// populated, `seed_registry` seeds from the pre-loaded handles.
#[test]
fn seeds_from_pre_loaded_handles() {
    let mut app = test_app();

    // Create typed assets.
    let (h_alpha, h_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha, h_beta)
    };

    // Insert RegistryHandles with loaded=true and handles already set.
    let mut rh = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    rh.loaded = true;
    rh.handles = vec![h_alpha, h_beta];
    app.insert_resource(rh);

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert_eq!(
        registry.entries.len(),
        2,
        "TestRegistry should contain 2 entries from pre-loaded handles"
    );
    assert!(registry.entries.contains_key("alpha"));
    assert!(registry.entries.contains_key("beta"));
}

/// `seed_registry` is idempotent via `Local<bool>` — once seeded,
/// subsequent updates do not re-seed.
#[test]
fn seed_registry_is_idempotent() {
    let mut app = test_app();

    // Create typed assets.
    let (h_alpha, h_beta) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        (h_alpha, h_beta)
    };

    // Insert RegistryHandles with loaded=true.
    let mut rh = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    rh.loaded = true;
    rh.handles = vec![h_alpha, h_beta];
    app.insert_resource(rh);

    // First tick — seeds.
    app.update();

    // Mutate registry externally.
    app.world_mut()
        .resource_mut::<TestRegistry>()
        .entries
        .insert("extra".to_string(), 999.0);

    // Second tick — should NOT re-seed.
    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        registry.entries.contains_key("extra"),
        "extra should still be present — seed_registry should not re-seed"
    );
}

/// When handles are loaded but one asset is missing from
/// `Assets<TestRegistryAsset>`, `seed_registry` does not seed.
#[test]
fn returns_zero_when_asset_missing() {
    let mut app = test_app();

    // Create only one asset, but register 2 handles.
    let (h_alpha, h_beta_dangling) = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
        let h_alpha = assets.add(TestRegistryAsset {
            name:  "alpha".to_string(),
            value: 1.0,
        });
        // Create a handle for beta, then remove the asset to simulate
        // missing.
        let h_beta = assets.add(TestRegistryAsset {
            name:  "beta".to_string(),
            value: 2.0,
        });
        let beta_id = h_beta.id();
        assets.remove(beta_id);
        (h_alpha, h_beta)
    };

    let mut rh = RegistryHandles::<TestRegistryAsset>::new(Handle::default());
    rh.loaded = true;
    rh.handles = vec![h_alpha, h_beta_dangling];
    app.insert_resource(rh);

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        registry.entries.is_empty(),
        "TestRegistry should remain empty when an asset is missing"
    );
}

/// When `LoadedFolder` is present but contains zero typed handles
/// (empty folder or all handles filtered by `try_typed`), the registry
/// should NOT be sealed empty. It should return zero progress and
/// allow a future retry.
#[test]
fn returns_zero_when_folder_has_no_typed_handles() {
    let mut app = test_app();

    // Create a LoadedFolder with no handles at all.
    let folder_handle = {
        let mut folders = app
            .world_mut()
            .resource_mut::<Assets<bevy::asset::LoadedFolder>>();
        folders.add(bevy::asset::LoadedFolder { handles: vec![] })
    };

    let rh = RegistryHandles::<TestRegistryAsset>::new(folder_handle);
    app.insert_resource(rh);

    app.update();

    let registry = app.world().resource::<TestRegistry>();
    assert!(
        registry.entries.is_empty(),
        "TestRegistry should remain empty (no typed handles)"
    );

    // Verify the registry is NOT permanently sealed — loaded should
    // be true (folder was resolved) but seeded should still be false.
    let handles = app.world().resource::<RegistryHandles<TestRegistryAsset>>();
    assert!(
        handles.loaded,
        "loaded should be true (folder was resolved)"
    );
    assert!(
        handles.handles.is_empty(),
        "handles should be empty (no typed handles in folder)"
    );
}
