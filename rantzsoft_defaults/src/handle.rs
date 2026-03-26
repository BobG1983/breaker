//! Typed handle wrapper for defaults assets.

use bevy::prelude::*;

/// A typed [`Resource`] wrapping a [`Handle`] to a defaults asset.
///
/// Stored as a resource so that seed and propagate systems can locate the
/// specific asset handle without relying on a monolithic collection struct.
#[derive(Resource, Debug)]
pub struct DefaultsHandle<D: Asset>(pub Handle<D>);

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Asset, TypePath, Deserialize, Clone, Debug)]
    struct TestAsset {
        value: f32,
    }

    /// `DefaultsHandle` wraps a `Handle` and the inner handle is accessible
    /// via `.0`.
    #[test]
    fn defaults_handle_wraps_handle() {
        let handle: Handle<TestAsset> = Handle::default();
        let defaults_handle = DefaultsHandle(handle.clone());
        assert_eq!(
            defaults_handle.0.id(),
            handle.id(),
            "inner handle should be accessible via .0"
        );
    }

    /// `DefaultsHandle` is a `Resource` (verifiable by inserting into an `App`).
    #[test]
    fn defaults_handle_is_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let handle: Handle<TestAsset> = Handle::default();
        app.insert_resource(DefaultsHandle(handle));
        assert!(
            app.world()
                .get_resource::<DefaultsHandle<TestAsset>>()
                .is_some(),
            "DefaultsHandle should be insertable as a Resource"
        );
    }
}
