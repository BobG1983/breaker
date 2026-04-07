//! Plugin registration for the lifecycle crate.

use std::sync::Mutex;

use bevy::{prelude::*, state::state::FreelyMutableState};

use crate::{
    dispatch::{dispatch_condition_routes, dispatch_message_routes},
    messages::{ChangeState, StateChanged, TransitionEnd, TransitionStart},
    routing_table::RoutingTable,
    transition::{
        messages::{TransitionOver, TransitionReady, TransitionRunComplete},
        orchestration::orchestrate_transitions,
        registry::TransitionRegistry,
        resources::{EndingTransition, RunningTransition, StartingTransition},
        traits::Transition,
    },
};

/// Lifecycle plugin — declarative state routing for Bevy 0.18.
///
/// Use the builder pattern to register state types. Each registered state
/// type gets a [`RoutingTable<S>`], lifecycle messages, and dispatch systems.
///
/// ```text
/// app.add_plugins(
///     RantzStateflowPlugin::new()
///         .register_state::<AppState>()
///         .register_state::<GameState>()
///         .register_state::<NodeState>()
/// );
/// ```
type RegistrationFn = Box<dyn FnOnce(&mut App) + Send + Sync>;

/// Lifecycle plugin implementation.
pub struct RantzStateflowPlugin {
    registrations: Mutex<Vec<RegistrationFn>>,
}

impl RantzStateflowPlugin {
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
    /// - [`TransitionStart<S>`] and [`TransitionEnd<S>`] messages
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
                    .add_message::<TransitionStart<S>>()
                    .add_message::<TransitionEnd<S>>()
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

    /// Register a custom transition effect type with its three phase systems.
    ///
    /// The three systems correspond to the three phases of a transition:
    /// - `start_system`: runs when `StartingTransition<T>` exists
    /// - `run_system`: runs when `RunningTransition<T>` exists
    /// - `end_system`: runs when `EndingTransition<T>` exists
    ///
    /// Each system is gated by `run_if(resource_exists::<MarkerResource<T>>)`.
    /// Systems are ordered `.before(orchestrate_transitions)` so phase
    /// transitions complete within each update cycle.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned (unrecoverable).
    #[must_use]
    pub fn register_custom_transition<T: Transition, M1, M2, M3>(
        self,
        start_system: impl IntoSystem<(), (), M1> + Send + Sync + 'static,
        run_system: impl IntoSystem<(), (), M2> + Send + Sync + 'static,
        end_system: impl IntoSystem<(), (), M3> + Send + Sync + 'static,
    ) -> Self {
        #[allow(clippy::expect_used, reason = "poisoned Mutex is unrecoverable")]
        self.registrations
            .lock()
            .expect("lifecycle plugin lock poisoned")
            .push(Box::new(move |app| {
                // Register the effect type in the TransitionRegistry
                app.world_mut()
                    .resource_mut::<TransitionRegistry>()
                    .register::<T>();

                // Register the three phase systems, gated on marker resources
                app.add_systems(
                    Update,
                    (
                        start_system
                            .run_if(resource_exists::<StartingTransition<T>>)
                            .before(orchestrate_transitions),
                        run_system
                            .run_if(resource_exists::<RunningTransition<T>>)
                            .before(orchestrate_transitions),
                        end_system
                            .run_if(resource_exists::<EndingTransition<T>>)
                            .before(orchestrate_transitions),
                    ),
                );
            }));
        self
    }
}

impl Default for RantzStateflowPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RantzStateflowPlugin {
    fn build(&self, app: &mut App) {
        // Register shared transition infrastructure
        app.init_resource::<TransitionRegistry>()
            .add_message::<TransitionReady>()
            .add_message::<TransitionRunComplete>()
            .add_message::<TransitionOver>()
            .add_systems(Update, orchestrate_transitions);

        // Register built-in transition effects
        crate::transition::effects::register_builtin_transitions(app);

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
