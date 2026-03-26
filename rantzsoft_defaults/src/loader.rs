//! Generic RON asset loader for defaults files.

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    reflect::TypePath,
};
use serde::de::DeserializeOwned;

/// A generic [`AssetLoader`] that deserializes RON files into `T`.
///
/// Each instance is configured with a set of file extensions it recognizes.
pub struct RonAssetLoader<T> {
    extensions: Vec<&'static str>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Asset + DeserializeOwned> TypePath for RonAssetLoader<T> {
    fn type_path() -> &'static str {
        // Use a static string since we can't easily construct a
        // generic-aware string at compile time.
        "rantzsoft_defaults::loader::RonAssetLoader<T>"
    }

    fn short_type_path() -> &'static str {
        "RonAssetLoader<T>"
    }
}

impl<T> RonAssetLoader<T> {
    /// Creates a new loader that recognizes the given file extensions.
    #[must_use]
    pub fn new(extensions: &[&'static str]) -> Self {
        Self {
            extensions: extensions.to_vec(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// Deserializes RON bytes into the target type.
///
/// This is the core deserialization logic used by [`RonAssetLoader`].
/// Exposed as a public function for testability.
pub fn deserialize_ron<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ron::error::SpannedError> {
    ron::de::from_bytes(bytes)
}

impl<T> AssetLoader for RonAssetLoader<T>
where
    T: Asset + DeserializeOwned + Send + Sync + 'static,
{
    type Asset = T;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(deserialize_ron(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Asset, TypePath, Deserialize, Clone, Debug, PartialEq)]
    struct TestAsset {
        value: f32,
    }

    /// `RonAssetLoader` deserializes valid RON bytes into the target type.
    ///
    /// Uses the `deserialize_ron` helper (which the loader calls internally)
    /// to verify correct deserialization without needing a full Bevy asset
    /// pipeline.
    #[test]
    fn loader_deserializes_valid_ron() {
        let ron_bytes = b"(value: 42.0)";
        let result = deserialize_ron::<TestAsset>(ron_bytes);
        let asset = result.expect("valid RON should deserialize successfully");
        assert!(
            (asset.value - 42.0).abs() < f32::EPSILON,
            "deserialized value should be 42.0, got {}",
            asset.value
        );
    }

    /// `RonAssetLoader` returns an error for invalid RON input.
    #[test]
    fn loader_rejects_invalid_ron() {
        let invalid_bytes = b"not valid ron {{{";
        let result = deserialize_ron::<TestAsset>(invalid_bytes);
        assert!(
            result.is_err(),
            "invalid RON bytes should produce an error"
        );
    }

    /// `RonAssetLoader::extensions` returns the extensions passed at
    /// construction time.
    #[test]
    fn loader_reports_correct_extensions() {
        let loader = RonAssetLoader::<TestAsset>::new(&["test.ron"]);
        assert_eq!(
            loader.extensions(),
            &["test.ron"],
            "extensions() should return the extensions passed to new()"
        );
    }
}
