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
