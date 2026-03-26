//! Re-exports the `GameConfig` derive macro from `rantzsoft_defaults_derive`.

// Allow proc-macro expansions to reference this crate by name (e.g.,
// `::rantzsoft_defaults::SeedableConfig`) even when expanding within
// this crate's own test module.
extern crate self as rantzsoft_defaults;

pub mod handle;
pub mod loader;
pub mod plugin;
pub mod prelude;
pub mod seedable;
pub mod systems;

pub use rantzsoft_defaults_derive::GameConfig;
pub use seedable::SeedableConfig;

#[cfg(test)]
mod tests {
    /// Verifies that `GameConfig` derive macro is accessible through the prelude path.
    ///
    /// The test imports `GameConfig` from `crate::prelude` (not the crate root)
    /// and uses it to derive a config struct from a simple defaults struct.
    /// This proves the prelude re-exports the derive macro correctly.
    #[test]
    fn prelude_reexports_game_config_derive_macro() {
        use bevy::prelude::*;
        use serde::Deserialize;

        use crate::prelude::GameConfig;

        #[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
        #[game_config(name = "TestPreludeConfig")]
        struct TestPreludeDefaults {
            value: f32,
        }

        impl Default for TestPreludeDefaults {
            fn default() -> Self {
                Self { value: 42.0 }
            }
        }

        // The derive macro generates TestPreludeConfig with Default
        let config = TestPreludeConfig::default();
        assert!(
            (config.value - 42.0).abs() < f32::EPSILON,
            "Config generated from prelude-imported GameConfig should have correct default value"
        );
    }

    /// Verifies that the original `rantzsoft_defaults::GameConfig` import path still works
    /// alongside the prelude path (both paths are valid).
    #[test]
    fn root_reexport_still_works_alongside_prelude() {
        use bevy::prelude::*;
        use serde::Deserialize;

        use crate::GameConfig;

        #[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
        #[game_config(name = "TestRootConfig")]
        struct TestRootDefaults {
            count: u32,
        }

        impl Default for TestRootDefaults {
            fn default() -> Self {
                Self { count: 7 }
            }
        }

        let config = TestRootConfig::default();
        assert_eq!(
            config.count, 7,
            "Config generated from root-imported GameConfig should have correct default value"
        );
    }

    // ── Reversed macro (defaults = "...") tests ────────────────────────

    /// Helper struct used by the reversed-macro tests.
    /// `GameConfig` with `defaults = "TestDefaults2"` should generate
    /// `TestDefaults2` from this config struct.
    #[derive(::bevy::prelude::Resource, Debug, Clone, PartialEq, crate::GameConfig)]
    #[game_config(defaults = "TestDefaults2")]
    struct TestConfig2 {
        speed: f32,
        count: u32,
    }

    impl Default for TestConfig2 {
        fn default() -> Self {
            Self {
                speed: 99.0,
                count: 3,
            }
        }
    }

    /// Reversed macro generates a `TestDefaults2` struct with matching fields.
    #[test]
    fn reversed_macro_generates_defaults_struct() {
        let defaults = TestDefaults2 {
            speed: 1.0,
            count: 2,
        };
        // If the struct exists and has the correct fields, this compiles and runs.
        assert!((defaults.speed - 1.0).abs() < f32::EPSILON);
        assert_eq!(defaults.count, 2);
    }

    /// `From<TestDefaults2>` for `TestConfig2` copies all fields.
    #[test]
    fn from_defaults_for_config() {
        let defaults = TestDefaults2 {
            speed: 42.0,
            count: 7,
        };
        let config = TestConfig2::from(defaults);
        assert!(
            (config.speed - 42.0).abs() < f32::EPSILON,
            "config.speed should be 42.0, got {}",
            config.speed
        );
        assert_eq!(config.count, 7, "config.count should be 7");
    }

    /// `From<TestConfig2>` for `TestDefaults2` copies all fields (reverse direction).
    #[test]
    fn from_config_for_defaults() {
        let config = TestConfig2 {
            speed: 42.0,
            count: 7,
        };
        let defaults = TestDefaults2::from(config);
        assert!(
            (defaults.speed - 42.0).abs() < f32::EPSILON,
            "defaults.speed should be 42.0, got {}",
            defaults.speed
        );
        assert_eq!(defaults.count, 7, "defaults.count should be 7");
    }

    /// `Default` for `TestDefaults2` delegates to `TestConfig2::default()`.
    #[test]
    fn defaults_default_delegates_to_config_default() {
        let defaults = TestDefaults2::default();
        assert!(
            (defaults.speed - 99.0).abs() < f32::EPSILON,
            "defaults.speed should be 99.0 (from TestConfig2::default()), got {}",
            defaults.speed
        );
        assert_eq!(
            defaults.count, 3,
            "defaults.count should be 3 (from TestConfig2::default())"
        );
    }

    /// `merge_from_defaults` overwrites all config fields from the defaults.
    #[test]
    fn merge_from_defaults_overwrites_all_fields() {
        let mut config = TestConfig2 {
            speed: 100.0,
            count: 5,
        };
        let defaults = TestDefaults2 {
            speed: 200.0,
            count: 10,
        };
        config.merge_from_defaults(&defaults);
        assert!(
            (config.speed - 200.0).abs() < f32::EPSILON,
            "config.speed should be 200.0 after merge, got {}",
            config.speed
        );
        assert_eq!(config.count, 10, "config.count should be 10 after merge");
    }

    // ── SeedableConfig tests ───────────────────────────────────────────

    /// Helper struct for `SeedableConfig` tests. Uses `path` and `ext` attributes.
    #[derive(::bevy::prelude::Resource, Debug, Clone, PartialEq, crate::GameConfig)]
    #[game_config(defaults = "TestDefaults3", path = "config/test.ron", ext = "test.ron")]
    struct TestConfig3 {
        value: f32,
    }

    impl Default for TestConfig3 {
        fn default() -> Self {
            Self { value: 0.0 }
        }
    }

    /// When `path` and `ext` are present, the macro generates a `SeedableConfig` impl
    /// on the defaults struct.
    #[test]
    fn seedable_config_generated_with_path_and_ext() {
        use crate::seedable::SeedableConfig;

        assert_eq!(
            <TestDefaults3 as SeedableConfig>::asset_path(),
            "config/test.ron",
            "asset_path() should return the path from the attribute"
        );
        assert_eq!(
            <TestDefaults3 as SeedableConfig>::extensions(),
            &["test.ron"],
            "extensions() should return the ext from the attribute"
        );
    }

    /// `SeedableConfig::Config` associated type points to the original config struct.
    #[test]
    fn seedable_config_associated_type_is_config() {
        use crate::seedable::SeedableConfig;

        // Verify the associated type by creating a Config from the Defaults.
        let defaults = TestDefaults3 { value: 55.0 };
        let config: <TestDefaults3 as SeedableConfig>::Config =
            <TestDefaults3 as SeedableConfig>::Config::from(defaults);
        assert!(
            (config.value - 55.0).abs() < f32::EPSILON,
            "SeedableConfig::Config should be TestConfig3"
        );
    }

    /// `SeedableConfig` re-export from the prelude is accessible.
    #[test]
    fn prelude_reexports_seedable_config() {
        use crate::prelude::SeedableConfig;

        // If this compiles, the prelude re-exports the trait.
        assert_eq!(
            <TestDefaults3 as SeedableConfig>::asset_path(),
            "config/test.ron"
        );
    }

    /// Prelude re-exports `DefaultsHandle`, `RonAssetLoader`, `DefaultsSystems`,
    /// `RantzDefaultsPlugin`, and `RantzDefaultsPluginBuilder`.
    #[test]
    fn prelude_reexports_new_types() {
        use crate::prelude::{
            DefaultsHandle, DefaultsSystems, RantzDefaultsPlugin, RantzDefaultsPluginBuilder,
            RonAssetLoader,
        };

        // If this compiles, the prelude re-exports all new types.
        // Exercise type names so they are considered used.
        let _ = std::mem::size_of::<DefaultsHandle<TestDefaults3>>();
        let _ = std::mem::size_of::<RonAssetLoader<TestDefaults3>>();
        let _ = std::mem::size_of::<DefaultsSystems>();
        let _ = std::mem::size_of::<RantzDefaultsPlugin>();
        let _ = std::mem::size_of::<RantzDefaultsPluginBuilder>();
    }
}
