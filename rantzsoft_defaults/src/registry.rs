//! Trait and types for folder-based registry asset loading.

use bevy::{asset::LoadedFolder, prelude::*};
use serde::de::DeserializeOwned;

/// A [`Resource`] populated from a folder of RON assets at boot time.
///
/// Unlike [`SeedableConfig`](crate::seedable::SeedableConfig) which loads a
/// single file, `SeedableRegistry` loads an entire folder and builds a lookup
/// from all files found within it.
///
/// # Required methods
///
/// - [`seed`](SeedableRegistry::seed) — destructively replace all entries
/// - [`update_single`](SeedableRegistry::update_single) — upsert one entry
///
/// # Provided methods
///
/// - [`update_all`](SeedableRegistry::update_all) — reset to default then seed
pub trait SeedableRegistry: Resource + Default + Send + Sync + 'static {
    /// The asset type loaded from each RON file in the registry folder.
    type Asset: Asset + DeserializeOwned + Clone + Send + Sync + 'static;

    /// Path to the folder containing registry assets (relative to `assets/`).
    fn asset_dir() -> &'static str;

    /// File extensions recognized for this asset type.
    fn extensions() -> &'static [&'static str];

    /// Populate the registry from loaded assets. Destructive — replaces all
    /// existing entries.
    fn seed(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]);

    /// Rebuild the registry from all assets. Default: reset to default then
    /// seed.
    fn update_all(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]) {
        *self = Self::default();
        self.seed(assets);
    }

    /// Update a single asset entry. Required — implementor defines upsert
    /// logic.
    fn update_single(&mut self, id: AssetId<Self::Asset>, asset: &Self::Asset);
}

/// Stores the folder handle and typed handles for a registry's assets.
///
/// Inserted at [`Startup`] by
/// [`init_registry_handles`](crate::systems::init_registry_handles).
/// The [`seed_registry`](crate::systems::seed_registry) system resolves the
/// folder, populates [`handles`](RegistryHandles::handles), and seeds the
/// registry resource.
#[derive(Resource)]
pub struct RegistryHandles<A: Asset> {
    /// Handle to the [`LoadedFolder`] for this registry's asset directory.
    pub folder: Handle<LoadedFolder>,
    /// Typed handles extracted from the loaded folder.
    pub handles: Vec<Handle<A>>,
    /// Whether the folder has been resolved and handles extracted.
    pub loaded: bool,
}

impl<A: Asset> RegistryHandles<A> {
    /// Creates a new `RegistryHandles` with the given folder handle, no typed
    /// handles, and `loaded` set to `false`.
    #[must_use]
    pub const fn new(folder: Handle<LoadedFolder>) -> Self {
        Self {
            folder,
            handles: Vec::new(),
            loaded: false,
        }
    }
}

#[cfg(test)]
mod tests;
