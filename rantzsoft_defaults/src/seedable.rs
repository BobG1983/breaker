//! Trait for config types that can be loaded from RON asset files.

use bevy::prelude::{Asset, Resource};

/// Marker trait for defaults types that are loadable from RON asset files.
///
/// Types implementing `SeedableConfig` declare their asset path and file
/// extensions, enabling the boot pipeline to load and seed them automatically.
///
/// # Type Parameters
///
/// The `Config` associated type is the `Resource` that gets seeded from the
/// loaded RON data at boot time.
pub trait SeedableConfig: Asset + Clone + Send + Sync + 'static {
    /// The `Resource` type seeded from this defaults asset.
    type Config: Resource + From<Self> + Send + Sync + 'static;

    /// Asset path used by `bevy_asset_loader` to load this RON file.
    fn asset_path() -> &'static str;

    /// File extensions recognized for this asset type.
    fn extensions() -> &'static [&'static str];
}
