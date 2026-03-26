//! Re-exports the `GameConfig` derive macro from `rantzsoft_defaults_derive`.

pub mod prelude;

pub use rantzsoft_defaults_derive::GameConfig;

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
}
