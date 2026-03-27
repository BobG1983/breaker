//! Plugin and builder for the `rantzsoft_defaults` config pipeline.

use std::sync::Mutex;

use bevy::{prelude::*, state::state::FreelyMutableState};

use crate::registry::SeedableRegistry;

/// System sets for the defaults pipeline.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefaultsSystems {
    /// Systems that seed config resources from loaded defaults assets.
    Seed,
    /// Systems that propagate modified defaults assets to config resources.
    PropagateDefaults,
}

/// Type-erased registration closure applied during plugin build.
type Registration = Box<dyn FnOnce(&mut App) + Send>;

/// Plugin that wires the config defaults pipeline.
///
/// Construct via [`RantzDefaultsPluginBuilder`].
pub struct RantzDefaultsPlugin {
    registrations: Mutex<Vec<Registration>>,
}

/// Builder for [`RantzDefaultsPlugin`].
///
/// Use [`add_config`](Self::add_config) to register each
/// [`SeedableConfig`](crate::seedable::SeedableConfig) type, and
/// [`add_registry`](Self::add_registry) to register each
/// [`SeedableRegistry`] type, then call [`build`](Self::build) to produce
/// the plugin.
pub struct RantzDefaultsPluginBuilder<S: FreelyMutableState + Clone> {
    loading_state: S,
    registrations: Vec<Registration>,
}

impl<S: FreelyMutableState + Clone> RantzDefaultsPluginBuilder<S> {
    /// Creates a new builder that gates seed systems on the given loading
    /// state.
    #[must_use]
    pub fn new(loading_state: S) -> Self {
        Self {
            loading_state,
            registrations: Vec::new(),
        }
    }

    /// Registers a config type with the plugin.
    ///
    /// This will set up the RON asset loader, startup handle initialization,
    /// seed system, and propagate system for the given
    /// [`SeedableConfig`](crate::seedable::SeedableConfig) type.
    #[must_use]
    pub fn add_config<D: crate::seedable::SeedableConfig + serde::de::DeserializeOwned>(
        mut self,
    ) -> Self {
        let loading_state = self.loading_state.clone();
        self.registrations.push(Box::new(move |app: &mut App| {
            app.init_asset::<D>();
            app.register_asset_loader(crate::loader::RonAssetLoader::<D>::new(D::extensions()));
            app.add_systems(Startup, crate::systems::init_defaults_handle::<D>);
            #[cfg(feature = "progress")]
            {
                use iyes_progress::prelude::ProgressReturningSystem;
                app.add_systems(
                    Update,
                    crate::systems::seed_config::<D>
                        .track_progress::<S>()
                        .in_set(DefaultsSystems::Seed)
                        .run_if(in_state(loading_state)),
                );
            }
            #[cfg(feature = "hot-reload")]
            app.add_systems(
                Update,
                crate::systems::propagate_defaults::<D>.in_set(DefaultsSystems::PropagateDefaults),
            );
        }));
        self
    }

    /// Registers a registry type with the plugin.
    ///
    /// This will set up the RON asset loader, startup folder handle
    /// initialization, init the registry resource, and register the seed
    /// system for the given [`SeedableRegistry`] type. The seed system
    /// only runs while the app is in the loading state passed to
    /// [`RantzDefaultsPluginBuilder::new`].
    #[must_use]
    pub fn add_registry<R: SeedableRegistry + 'static>(mut self) -> Self
    where
        R::Asset: serde::de::DeserializeOwned,
    {
        let loading_state = self.loading_state.clone();
        self.registrations.push(Box::new(move |app: &mut App| {
            app.init_asset::<R::Asset>();
            app.register_asset_loader(crate::loader::RonAssetLoader::<R::Asset>::new(
                R::extensions(),
            ));
            app.add_systems(Startup, crate::systems::init_registry_handles::<R>);
            app.init_resource::<R>();
            #[cfg(feature = "progress")]
            {
                use iyes_progress::prelude::ProgressReturningSystem;
                app.add_systems(
                    Update,
                    crate::systems::seed_registry::<R>
                        .track_progress::<S>()
                        .in_set(DefaultsSystems::Seed)
                        .run_if(in_state(loading_state)),
                );
            }
            #[cfg(feature = "hot-reload")]
            app.add_systems(
                Update,
                crate::systems::propagate_registry::<R>.in_set(DefaultsSystems::PropagateDefaults),
            );
        }));
        self
    }

    /// Builds the plugin from the accumulated registrations.
    #[must_use]
    pub fn build(self) -> RantzDefaultsPlugin {
        RantzDefaultsPlugin {
            registrations: Mutex::new(self.registrations),
        }
    }
}

impl Plugin for RantzDefaultsPlugin {
    fn build(&self, app: &mut App) {
        #[allow(clippy::expect_used, reason = "poisoned Mutex is unrecoverable")]
        let mut registrations = self
            .registrations
            .lock()
            .expect("defaults plugin lock poisoned");
        for registration in registrations.drain(..) {
            registration(app);
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests panic on failure")]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::*;
    use crate::{handle::DefaultsHandle, registry::RegistryHandles};

    /// Test state for builder generic tests.
    #[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    enum TestState {
        #[default]
        Loading,
        Running,
    }

    /// Test config type generated by the `GameConfig` derive macro.
    #[derive(Resource, Debug, Clone, PartialEq, Default, crate::GameConfig)]
    #[game_config(defaults = "TestDefaults", path = "config/test.ron", ext = "test.ron")]
    struct TestConfig {
        value: f32,
    }

    /// Test asset type for registry builder tests.
    #[derive(Asset, TypePath, Deserialize, Clone, Debug)]
    struct TestRegistryAsset {
        name: String,
        value: f32,
    }

    /// Test registry type for builder tests.
    #[derive(Resource, Default, Debug)]
    struct TestRegistry {
        entries: HashMap<String, f32>,
    }

    impl crate::registry::SeedableRegistry for TestRegistry {
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

    /// `DefaultsSystems::Seed` and `DefaultsSystems::PropagateDefaults` are
    /// distinct variants that implement `SystemSet`.
    #[test]
    fn defaults_systems_variants_are_distinct() {
        let seed = DefaultsSystems::Seed;
        let propagate = DefaultsSystems::PropagateDefaults;
        assert_ne!(
            seed, propagate,
            "Seed and PropagateDefaults should be distinct system sets"
        );
    }

    /// After adding a plugin built with `add_config`, the asset loader
    /// for the registered defaults type should be available (the plugin
    /// registers it).
    #[test]
    fn plugin_registers_asset_loader() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(plugin);

        // After the plugin is added, we should be able to get the asset
        // collection for TestDefaults (the loader registered via init_asset).
        // If the plugin properly registers the loader, the Assets<TestDefaults>
        // resource should exist.
        app.update();

        assert!(
            app.world().get_resource::<Assets<TestDefaults>>().is_some(),
            "Assets<TestDefaults> should be registered by the plugin"
        );
    }

    /// After startup, the plugin should insert `DefaultsHandle<TestDefaults>`.
    #[test]
    fn plugin_inserts_defaults_handle_on_startup() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(plugin);

        // Startup systems run on first update.
        app.update();

        assert!(
            app.world()
                .get_resource::<DefaultsHandle<TestDefaults>>()
                .is_some(),
            "DefaultsHandle<TestDefaults> should exist after plugin startup"
        );
    }

    /// The seed system should run during the loading phase. We verify this
    /// indirectly: after inserting a loaded asset and its handle, the config
    /// resource should be seeded after an update cycle.
    ///
    /// The test manually registers the asset type and inserts a loaded asset.
    /// The plugin should register the seed system that reads it.
    #[test]
    fn plugin_seed_system_seeds_config() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ));
        app.init_state::<TestState>();
        app.add_plugins(
            iyes_progress::prelude::ProgressPlugin::<TestState>::new()
                .with_state_transition(TestState::Loading, TestState::Running),
        );
        // Manually register asset type so we can add assets for the seed
        // system. The plugin should also register it, but we need it early
        // for setup.
        app.init_asset::<TestDefaults>();
        app.add_plugins(plugin);

        // First update runs startup (inserts DefaultsHandle).
        app.update();

        // Insert a loaded asset and update the handle to point to it.
        let defaults = TestDefaults { value: 42.0 };
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
            assets.add(defaults)
        };
        app.world_mut().insert_resource(DefaultsHandle(handle));

        // Run update to trigger seed system.
        app.update();

        let config = app
            .world()
            .get_resource::<TestConfig>()
            .expect("TestConfig should be seeded by the plugin");
        assert!(
            (config.value - 42.0).abs() < f32::EPSILON,
            "TestConfig.value should be 42.0, got {}",
            config.value
        );
    }

    // ── Behavior 12: Generic builder constructs with state type ──────

    /// `RantzDefaultsPluginBuilder` constructs with a state type, builds a
    /// plugin, and the app runs without panic.
    #[test]
    fn generic_builder_constructs_with_state_type() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading).build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(plugin);

        // Should not panic.
        app.update();
    }

    // ── Behavior 13: add_config registers asset loader and startup handle

    /// `add_config` registers the asset type and inserts `DefaultsHandle` at
    /// startup.
    #[test]
    fn add_config_registers_asset_loader_and_startup_handle() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(plugin);

        app.update();

        assert!(
            app.world()
                .get_resource::<DefaultsHandle<TestDefaults>>()
                .is_some(),
            "DefaultsHandle<TestDefaults> should exist after add_config startup"
        );
        assert!(
            app.world().get_resource::<Assets<TestDefaults>>().is_some(),
            "Assets<TestDefaults> should be registered after add_config"
        );
    }

    // ── Behavior 14: add_registry registers asset type, handles, resource

    /// `add_registry` registers the asset type, inserts
    /// `RegistryHandles<TestRegistryAsset>`, inits `TestRegistry`, and
    /// registers `Assets<TestRegistryAsset>`.
    #[test]
    fn add_registry_registers_asset_type_and_resources() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_registry::<TestRegistry>()
            .build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(plugin);

        app.update();

        assert!(
            app.world()
                .get_resource::<RegistryHandles<TestRegistryAsset>>()
                .is_some(),
            "RegistryHandles<TestRegistryAsset> should exist after add_registry startup"
        );
        assert!(
            app.world().get_resource::<TestRegistry>().is_some(),
            "TestRegistry should be initialized after add_registry"
        );
        assert!(
            app.world()
                .get_resource::<Assets<TestRegistryAsset>>()
                .is_some(),
            "Assets<TestRegistryAsset> should be registered after add_registry"
        );
    }

    // ── Behavior 15: add_config seed only runs in loading state ──────

    /// When app state is `TestState::Running` (not `Loading`), the seed
    /// system registered by `add_config` does not seed the config resource.
    #[test]
    fn add_config_seed_only_runs_in_loading_state() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<TestState>();
        app.init_asset::<TestDefaults>();
        app.add_plugins(plugin);

        // Start in Running state (not Loading).
        app.world_mut()
            .resource_mut::<NextState<TestState>>()
            .set(TestState::Running);
        app.update();

        // Insert a loaded asset and handle.
        let defaults = TestDefaults { value: 42.0 };
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
            assets.add(defaults)
        };
        app.world_mut().insert_resource(DefaultsHandle(handle));

        // Run another update — seed should NOT run because state is Running.
        app.update();

        assert!(
            app.world().get_resource::<TestConfig>().is_none(),
            "TestConfig should NOT exist when app state is Running (not Loading)"
        );
    }

    // ── Behavior: add_config wires propagate_defaults (hot-reload) ────

    /// When the `hot-reload` feature is enabled, `add_config` wires
    /// `propagate_defaults` so that modifying the asset updates the config.
    #[cfg(all(feature = "hot-reload", feature = "progress"))]
    #[test]
    fn add_config_wires_propagate_defaults_on_hot_reload() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_config::<TestDefaults>()
            .build();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ));
        app.init_state::<TestState>();
        app.init_asset::<TestDefaults>();
        app.add_plugins(
            iyes_progress::prelude::ProgressPlugin::<TestState>::new()
                .with_state_transition(TestState::Loading, TestState::Running),
        );
        app.add_plugins(plugin);

        // First update runs startup.
        app.update();

        // Insert a loaded asset and update the handle.
        let defaults = TestDefaults { value: 10.0 };
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
            assets.add(defaults)
        };
        app.world_mut()
            .insert_resource(DefaultsHandle(handle.clone()));

        // Seed the config initially.
        app.update();

        // Verify initial seeding.
        let config = app
            .world()
            .get_resource::<TestConfig>()
            .expect("TestConfig should be seeded");
        assert!(
            (config.value - 10.0).abs() < f32::EPSILON,
            "initial value should be 10.0, got {}",
            config.value
        );

        // Let Added event settle.
        app.update();

        // Trigger Modified by mutating the asset.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<TestDefaults>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.value = 77.0;
        }

        // PostUpdate flushes Modified, First rotates buffer.
        app.update();
        app.update();

        let config = app.world().resource::<TestConfig>();
        assert!(
            (config.value - 77.0).abs() < f32::EPSILON,
            "TestConfig.value should be 77.0 after Modified event via builder wiring, got {}",
            config.value
        );
    }

    // ── Behavior: add_registry wires propagate_registry (hot-reload) ──

    /// When the `hot-reload` feature is enabled, `add_registry` wires
    /// `propagate_registry` so that modifying a registry asset rebuilds
    /// the registry.
    #[cfg(all(feature = "hot-reload", feature = "progress"))]
    #[test]
    fn add_registry_wires_propagate_registry_on_hot_reload() {
        let plugin = RantzDefaultsPluginBuilder::<TestState>::new(TestState::Loading)
            .add_registry::<TestRegistry>()
            .build();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ));
        app.init_state::<TestState>();
        app.init_asset::<TestRegistryAsset>();
        app.init_asset::<bevy::asset::LoadedFolder>();
        app.add_plugins(
            iyes_progress::prelude::ProgressPlugin::<TestState>::new()
                .with_state_transition(TestState::Loading, TestState::Running),
        );
        app.add_plugins(plugin);

        // First update runs startup.
        app.update();

        // Create a typed asset.
        let h_alpha = {
            let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
            assets.add(TestRegistryAsset {
                name: "alpha".to_string(),
                value: 1.0,
            })
        };

        // Set up RegistryHandles as if already seeded.
        let mut rh = crate::registry::RegistryHandles::<TestRegistryAsset>::new(Handle::default());
        rh.loaded = true;
        rh.handles = vec![h_alpha.clone()];
        app.insert_resource(rh);

        // Pre-seed the registry.
        app.world_mut()
            .resource_mut::<TestRegistry>()
            .entries
            .insert("alpha".to_string(), 1.0);

        // Let Added settle.
        app.update();
        app.update();

        // Trigger Modified by mutating the asset.
        {
            let mut assets = app.world_mut().resource_mut::<Assets<TestRegistryAsset>>();
            let asset = assets.get_mut(h_alpha.id()).expect("asset should exist");
            asset.value = 42.0;
        }

        // PostUpdate flushes Modified, First rotates buffer, Update runs
        // system.
        app.update();
        app.update();

        let registry = app.world().resource::<TestRegistry>();
        assert!(
            (registry.entries["alpha"] - 42.0).abs() < f32::EPSILON,
            "alpha should be 42.0 after Modified event via builder wiring, got {}",
            registry.entries["alpha"]
        );
    }
}
