// ── seed_config tests ────────────────────────────────────────────

use bevy::prelude::*;

use super::helpers::{TestConfig, TestDefaults};
use crate::{handle::DefaultsHandle, systems::*};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TestDefaults>()
        .add_systems(Update, seed_config::<TestDefaults>.map(drop));
    app
}

/// Without a `DefaultsHandle` resource, `seed_config` reports zero
/// progress and does not insert the `Config` resource.
#[test]
fn returns_zero_progress_without_handle() {
    let mut app = test_app();
    app.update();
    assert!(
        app.world().get_resource::<TestConfig>().is_none(),
        "TestConfig should not exist when no DefaultsHandle is present"
    );
}

/// When `DefaultsHandle` exists but the asset is not yet loaded,
/// `seed_config` reports zero progress.
#[test]
fn returns_zero_progress_when_asset_not_loaded() {
    let mut app = test_app();

    // Insert a handle pointing to a non-existent asset.
    let handle: Handle<TestDefaults> = Handle::default();
    app.world_mut().insert_resource(DefaultsHandle(handle));

    app.update();

    assert!(
        app.world().get_resource::<TestConfig>().is_none(),
        "TestConfig should not exist when the asset is not loaded"
    );
}

/// When the defaults asset is loaded, `seed_config` inserts the
/// corresponding `Config` resource and reports done progress.
#[test]
fn inserts_config_when_asset_loaded() {
    let mut app = test_app();

    let defaults = TestDefaults { value: 99.0 };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
        assets.add(defaults)
    };
    app.world_mut().insert_resource(DefaultsHandle(handle));

    app.update();

    let config = app
        .world()
        .get_resource::<TestConfig>()
        .expect("TestConfig should be inserted after seed");
    assert!(
        (config.value - 99.0).abs() < f32::EPSILON,
        "TestConfig.value should be 99.0 from the loaded defaults, got {}",
        config.value
    );
}

/// After seeding once, running `seed_config` again does not re-insert
/// the resource (idempotent). Reports done progress on subsequent ticks.
#[test]
fn seed_is_idempotent() {
    let mut app = test_app();

    let defaults = TestDefaults { value: 99.0 };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
        assets.add(defaults)
    };
    app.world_mut().insert_resource(DefaultsHandle(handle));

    // First tick — seeds the config.
    app.update();

    // Mutate the config directly to detect re-insertion.
    app.world_mut().resource_mut::<TestConfig>().value = 1.0;

    // Second tick — should NOT re-insert (value stays at 1.0).
    app.update();

    let config = app.world().resource::<TestConfig>();
    assert!(
        (config.value - 1.0).abs() < f32::EPSILON,
        "seed_config should be idempotent — value should remain 1.0, got {}",
        config.value
    );
}
