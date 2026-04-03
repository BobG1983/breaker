//! Plugin registration for the lifecycle crate.

use std::sync::Mutex;

use bevy::{prelude::*, state::state::FreelyMutableState};

use crate::{
    dispatch::{dispatch_condition_routes, dispatch_message_routes},
    messages::{ChangeState, StateChanged},
    routing_table::RoutingTable,
};

/// Lifecycle plugin — declarative state routing for Bevy 0.18.
///
/// Use the builder pattern to register state types. Each registered state
/// type gets a [`RoutingTable<S>`], lifecycle messages, and dispatch systems.
///
/// ```text
/// app.add_plugins(
///     RantzLifecyclePlugin::new()
///         .register_state::<AppState>()
///         .register_state::<GameState>()
///         .register_state::<NodeState>()
/// );
/// ```
type RegistrationFn = Box<dyn FnOnce(&mut App) + Send + Sync>;

/// Lifecycle plugin implementation.
pub struct RantzLifecyclePlugin {
    registrations: Mutex<Vec<RegistrationFn>>,
}

impl RantzLifecyclePlugin {
    /// Create a new lifecycle plugin builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registrations: Mutex::new(Vec::new()),
        }
    }

    /// Register all lifecycle infrastructure for state type `S`:
    /// - [`RoutingTable<S>`] resource
    /// - [`ChangeState<S>`] and [`StateChanged<S>`] messages
    /// - Message-triggered and condition-triggered dispatch systems
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned (unrecoverable).
    #[must_use]
    pub fn register_state<S: FreelyMutableState + Eq + std::hash::Hash + Copy>(self) -> Self {
        #[allow(clippy::expect_used, reason = "poisoned Mutex is unrecoverable")]
        self.registrations
            .lock()
            .expect("lifecycle plugin lock poisoned")
            .push(Box::new(|app| {
                app.init_resource::<RoutingTable<S>>()
                    .add_message::<ChangeState<S>>()
                    .add_message::<StateChanged<S>>()
                    .add_systems(
                        Update,
                        (
                            dispatch_message_routes::<S>.run_if(on_message::<ChangeState<S>>),
                            dispatch_condition_routes::<S>,
                        ),
                    );
            }));
        self
    }
}

impl Default for RantzLifecyclePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RantzLifecyclePlugin {
    fn build(&self, app: &mut App) {
        #[allow(clippy::expect_used, reason = "poisoned Mutex is unrecoverable")]
        let mut registrations = self
            .registrations
            .lock()
            .expect("lifecycle plugin lock poisoned");
        for registration in registrations.drain(..) {
            registration(app);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{Route, RoutingTable, routing_table::RoutingTableAppExt};

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum AppState {
        #[default]
        Loading,
        Game,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(AppState = AppState::Game)]
    enum GameState {
        #[default]
        Menu,
    }

    #[test]
    fn plugin_builds_and_registers_state_types() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_plugins(
                RantzLifecyclePlugin::new()
                    .register_state::<AppState>()
                    .register_state::<GameState>(),
            );
        app.update();

        // Routing tables should exist
        assert!(app.world().contains_resource::<RoutingTable<AppState>>());
        assert!(app.world().contains_resource::<RoutingTable<GameState>>());
    }

    #[test]
    fn plugin_dispatch_works_end_to_end() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_plugins(
                RantzLifecyclePlugin::new()
                    .register_state::<AppState>()
                    .register_state::<GameState>(),
            );

        // Add route: Loading → Game
        app.add_route(Route::from(AppState::Loading).to(AppState::Game));

        app.update();
        assert_eq!(
            **app.world().resource::<State<AppState>>(),
            AppState::Loading
        );

        // Send ChangeState — plugin's dispatch should route Loading → Game
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<AppState>>>()
            .write(ChangeState::new());
        app.update();
        app.update();

        assert_eq!(**app.world().resource::<State<AppState>>(), AppState::Game,);
    }

    #[test]
    fn plugin_condition_dispatch_works() {
        #[derive(Resource)]
        struct Ready(bool);

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .insert_resource(Ready(false))
            .add_plugins(RantzLifecyclePlugin::new().register_state::<AppState>());

        app.add_route(
            Route::from(AppState::Loading)
                .to(AppState::Game)
                .when(|world| world.resource::<Ready>().0),
        );

        app.update();
        app.update();
        // Not ready yet
        assert_eq!(
            **app.world().resource::<State<AppState>>(),
            AppState::Loading
        );

        app.world_mut().resource_mut::<Ready>().0 = true;
        app.update();
        app.update();

        assert_eq!(**app.world().resource::<State<AppState>>(), AppState::Game,);
    }

    #[test]
    fn plugin_sends_state_changed_message() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_plugins(RantzLifecyclePlugin::new().register_state::<AppState>());

        app.add_route(Route::from(AppState::Loading).to(AppState::Game));
        app.update();

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<AppState>>>()
            .write(ChangeState::new());
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<StateChanged<AppState>>>();
        let changed: Vec<_> = msgs.iter_current_update_messages().collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].from, AppState::Loading);
        assert_eq!(changed[0].to, AppState::Game);
    }
}
