use bevy::prelude::*;
#[cfg(feature = "progress")]
use iyes_progress::prelude::*;

use crate::{
    handle::DefaultsHandle,
    registry::{RegistryHandles, SeedableRegistry},
    seedable::SeedableConfig,
};

/// Seeds a `Config` resource from the loaded defaults asset.
///
/// Returns [`Progress`] indicating whether the config has been seeded.
/// Uses a [`Local<bool>`] to ensure idempotency — once seeded, always reports
/// done without re-inserting.
#[cfg(feature = "progress")]
#[must_use]
pub fn seed_config<D: SeedableConfig>(
    handle: Option<Res<DefaultsHandle<D>>>,
    assets: Res<Assets<D>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }
    let Some(handle) = handle else {
        return Progress { done: 0, total: 1 };
    };
    let Some(defaults) = assets.get(handle.0.id()) else {
        return Progress { done: 0, total: 1 };
    };
    commands.insert_resource::<D::Config>(defaults.clone().into());
    *seeded = true;
    Progress { done: 1, total: 1 }
}

/// Watches for [`AssetEvent::Modified`] on the defaults asset and re-seeds
/// the `Config` resource with updated values.
#[cfg(feature = "hot-reload")]
pub fn propagate_defaults<D: SeedableConfig>(
    mut events: MessageReader<AssetEvent<D>>,
    handle: Res<DefaultsHandle<D>>,
    assets: Res<Assets<D>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.is_modified(handle.0.id())
            && let Some(defaults) = assets.get(handle.0.id())
        {
            commands.insert_resource::<D::Config>(defaults.clone().into());
        }
    }
}

/// Watches for [`AssetEvent::Modified`] on registry assets and rebuilds the
/// registry with updated values.
#[cfg(feature = "hot-reload")]
pub fn propagate_registry<R: SeedableRegistry>(
    mut events: MessageReader<AssetEvent<R::Asset>>,
    mut registry: ResMut<R>,
    handles: Res<RegistryHandles<R::Asset>>,
    assets: Res<Assets<R::Asset>>,
) {
    let any_modified = events
        .read()
        .any(|event| handles.handles.iter().any(|h| event.is_modified(h.id())));

    if !any_modified {
        return;
    }

    let collected: Vec<_> = handles
        .handles
        .iter()
        .filter_map(|h| assets.get(h.id()).map(|a| (h.id(), a.clone())))
        .collect();

    registry.update_all(&collected);
}

/// Startup system that loads the defaults asset and inserts a
/// [`DefaultsHandle`] resource.
pub fn init_defaults_handle<D: SeedableConfig>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle = asset_server.load::<D>(D::asset_path());
    commands.insert_resource(DefaultsHandle::<D>(handle));
}

/// Startup system that loads a registry folder and inserts
/// [`RegistryHandles`] for it.
pub fn init_registry_handles<R: SeedableRegistry>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let folder = asset_server.load_folder(R::asset_dir());
    commands.insert_resource(RegistryHandles::<R::Asset>::new(folder));
}

/// Seeds a registry [`Resource`] from a folder of loaded assets.
///
/// Returns [`Progress`] indicating whether the registry has been seeded.
/// Uses a [`Local<bool>`] to ensure idempotency — once seeded, always
/// reports done without re-seeding.
#[cfg(feature = "progress")]
#[must_use]
pub fn seed_registry<R: SeedableRegistry>(
    mut registry: ResMut<R>,
    handles: Option<ResMut<RegistryHandles<R::Asset>>>,
    loaded_folders: Res<Assets<bevy::asset::LoadedFolder>>,
    assets: Res<Assets<R::Asset>>,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(mut handles) = handles else {
        return Progress { done: 0, total: 1 };
    };

    if !handles.loaded {
        let Some(folder) = loaded_folders.get(&handles.folder) else {
            return Progress { done: 0, total: 1 };
        };
        handles.handles = folder
            .handles
            .iter()
            .filter_map(|untyped| untyped.clone().try_typed::<R::Asset>().ok())
            .collect();
        handles.loaded = true;
    }

    // An empty folder (or one where try_typed filtered everything) is likely
    // a misconfiguration — treat as "not ready" rather than sealing the
    // registry empty forever.
    if handles.handles.is_empty() {
        return Progress { done: 0, total: 1 };
    }

    let mut collected = Vec::with_capacity(handles.handles.len());
    for handle in &handles.handles {
        let Some(asset) = assets.get(handle.id()) else {
            return Progress { done: 0, total: 1 };
        };
        collected.push((handle.id(), asset.clone()));
    }

    registry.seed(&collected);
    *seeded = true;
    Progress { done: 1, total: 1 }
}
