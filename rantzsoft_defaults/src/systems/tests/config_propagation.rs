// ── propagate_defaults tests ─────────────────────────────────────

use bevy::prelude::*;

use super::helpers::{TestConfig, TestDefaults};
use crate::{handle::DefaultsHandle, systems::*};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<TestDefaults>()
        .init_resource::<TestConfig>()
        .add_systems(Update, propagate_defaults::<TestDefaults>);
    app
}

/// When only an `Added` event fires (no `Modified`), the config
/// remains unchanged.
#[test]
fn config_unchanged_when_no_modified_event() {
    let mut app = test_app();

    let defaults = TestDefaults { value: 10.0 };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
        assets.add(defaults)
    };
    app.world_mut().insert_resource(DefaultsHandle(handle));

    // Two updates: PostUpdate flushes Added, First rotates buffer.
    app.update();
    app.update();

    let config = app.world().resource::<TestConfig>();
    assert!(
        (config.value - 0.0).abs() < f32::EPSILON,
        "config should remain at default (0.0) when only Added fires, got {}",
        config.value
    );
}

/// When a `Modified` event fires after mutating the asset,
/// `propagate_defaults` replaces the `Config` resource with new
/// values.
#[test]
fn config_replaced_on_modified_event() {
    let mut app = test_app();

    let defaults = TestDefaults { value: 10.0 };
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
        assets.add(defaults)
    };
    app.world_mut()
        .insert_resource(DefaultsHandle(handle.clone()));

    // Let Added event settle.
    app.update();
    app.update();

    // Trigger Modified by mutating the asset.
    {
        let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
        let asset = assets.get_mut(handle.id()).expect("asset should exist");
        asset.value = 77.0;
    }

    // PostUpdate flushes Modified, First rotates buffer, Update runs
    // system.
    app.update();
    app.update();

    let config = app.world().resource::<TestConfig>();
    assert!(
        (config.value - 77.0).abs() < f32::EPSILON,
        "TestConfig.value should be 77.0 after Modified event, got {}",
        config.value
    );
}
