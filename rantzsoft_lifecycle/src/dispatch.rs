//! Dispatch systems — route execution triggered by messages or conditions.
//!
//! Two exclusive systems per registered state type:
//! - [`dispatch_message_routes`]: fires when [`ChangeState<S>`] is received
//! - [`dispatch_condition_routes`]: polls `when()` conditions each frame
//!
//! Both use `resource_scope` to extract the routing table, giving the
//! dynamic closures `&World` access without borrow conflicts.

use bevy::{prelude::*, state::state::FreelyMutableState};
use tracing::warn;

use crate::{
    messages::{ChangeState, StateChanged},
    route::{DestinationKind, TriggerKind},
    routing_table::RoutingTable,
};

/// Message-triggered dispatch. Reads current `State<S>`, looks up the route
/// in [`RoutingTable<S>`], resolves the destination (static or dynamic via
/// `&World`), and applies `NextState<S>::set()`.
///
/// Gated by `run_if(on_message::<ChangeState<S>>())` — zero per-frame cost
/// when idle. Processes the first message only; warns on duplicates.
pub fn dispatch_message_routes<S: FreelyMutableState + Eq + std::hash::Hash + Copy>(
    world: &mut World,
) {
    // Count messages (the run_if already confirmed at least one exists)
    let count = world
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<S>>>()
        .drain()
        .count();

    if count > 1 {
        warn!(
            "Multiple ChangeState<{}> messages in one frame ({count}) — processing first only",
            std::any::type_name::<S>(),
        );
    }

    let current = *world.resource::<State<S>>().get();

    world.resource_scope(|world, table: Mut<RoutingTable<S>>| {
        let Some(route) = table.routes.get(&current) else {
            warn!(
                "No route registered for {current:?} in RoutingTable<{}> — ignoring ChangeState",
                std::any::type_name::<S>(),
            );
            return;
        };

        // Skip condition-triggered routes
        if matches!(route.trigger, TriggerKind::Condition(_)) {
            warn!(
                "ChangeState<{}> received for condition-triggered route from {current:?} — ignoring",
                std::any::type_name::<S>(),
            );
            return;
        }

        let destination = resolve_destination(&route.destination, current, world);
        let Some(destination) = destination else { return };

        world.resource_mut::<NextState<S>>().set(destination);
        world
            .resource_mut::<bevy::ecs::message::Messages<StateChanged<S>>>()
            .write(StateChanged {
                from: current,
                to: destination,
            });
    });
}

/// Condition-triggered dispatch. Runs every frame in `Update`. Evaluates
/// the `when()` condition for the route matching the current state. If
/// the condition returns `true`, executes the route.
///
/// Near-zero per-frame cost: checks at most one route per state type.
pub fn dispatch_condition_routes<S: FreelyMutableState + Eq + std::hash::Hash + Copy>(
    world: &mut World,
) {
    let Some(state) = world.get_resource::<State<S>>() else {
        return; // SubState not active (parent in wrong state)
    };
    let current = *state.get();

    world.resource_scope(|world, table: Mut<RoutingTable<S>>| {
        let Some(route) = table.routes.get(&current) else {
            return;
        };

        let TriggerKind::Condition(ref when_fn) = route.trigger else {
            return;
        };

        if !when_fn(world) {
            return;
        }

        let destination = resolve_destination(&route.destination, current, world);
        let Some(destination) = destination else {
            return;
        };

        world.resource_mut::<NextState<S>>().set(destination);
        world
            .resource_mut::<bevy::ecs::message::Messages<StateChanged<S>>>()
            .write(StateChanged {
                from: current,
                to: destination,
            });
    });
}

/// Resolve the destination from a route, logging a warning on `None`.
fn resolve_destination<S: States + Copy>(
    destination: &DestinationKind<S>,
    current: S,
    world: &World,
) -> Option<S> {
    match destination {
        DestinationKind::None => {
            warn!("Route from {current:?} has no destination — ignoring",);
            None
        }
        DestinationKind::Static(s) => Some(*s),
        DestinationKind::Dynamic(f) => Some(f(world)),
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::Route;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        Loading,
        AnimateIn,
        Playing,
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_message::<ChangeState<TestState>>()
            .add_message::<StateChanged<TestState>>()
            .init_resource::<RoutingTable<TestState>>()
            .add_systems(
                Update,
                dispatch_message_routes::<TestState>.run_if(on_message::<ChangeState<TestState>>),
            );
        app
    }

    fn send_change_state(app: &mut App) {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
            .write(ChangeState::new());
    }

    // --- Message-triggered dispatch ---

    #[test]
    fn dispatch_transitions_via_static_route() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
            .ok();

        app.update();

        send_change_state(&mut app);
        app.update(); // dispatch fires, sets NextState
        app.update(); // state transition applies

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::AnimateIn,
        );
    }

    #[test]
    fn dispatch_transitions_via_dynamic_route() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(Route::from(TestState::Loading).to_dynamic(|_world| TestState::Playing))
            .ok();

        app.update();

        send_change_state(&mut app);
        app.update();
        app.update();

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Playing,
        );
    }

    #[test]
    fn dispatch_sends_state_changed_message() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
            .ok();

        app.update();

        send_change_state(&mut app);
        app.update(); // dispatch fires

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<StateChanged<TestState>>>();
        let changed: Vec<_> = msgs.iter_current_update_messages().collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].from, TestState::Loading);
        assert_eq!(changed[0].to, TestState::AnimateIn);
    }

    #[test]
    fn dispatch_does_nothing_without_route() {
        let mut app = test_app();
        app.update();

        send_change_state(&mut app);
        app.update();
        app.update();

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );
    }

    #[test]
    fn dispatch_does_nothing_without_message() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
            .ok();

        app.update();
        app.update();

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );
    }

    #[test]
    fn dispatch_skips_condition_triggered_routes() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(
                Route::from(TestState::Loading)
                    .to(TestState::AnimateIn)
                    .when(|_| true),
            )
            .ok();

        app.update();

        send_change_state(&mut app);
        app.update();
        app.update();

        // Condition route should be skipped by message dispatch
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );
    }

    #[test]
    fn dispatch_chains_through_multiple_routes() {
        let mut app = test_app();
        {
            let mut table = app.world_mut().resource_mut::<RoutingTable<TestState>>();
            table
                .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
                .ok();
            table
                .add(Route::from(TestState::AnimateIn).to(TestState::Playing))
                .ok();
        }

        app.update();

        // First: Loading → AnimateIn
        send_change_state(&mut app);
        app.update();
        app.update();
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::AnimateIn,
        );

        // Second: AnimateIn → Playing
        send_change_state(&mut app);
        app.update();
        app.update();
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Playing,
        );
    }

    // --- Condition-triggered dispatch ---

    #[test]
    fn condition_dispatch_fires_when_condition_true() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_message::<StateChanged<TestState>>()
            .init_resource::<RoutingTable<TestState>>()
            .add_systems(Update, dispatch_condition_routes::<TestState>);

        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(
                Route::from(TestState::Loading)
                    .to(TestState::AnimateIn)
                    .when(|_world| true),
            )
            .ok();

        app.update(); // initial
        app.update(); // condition evaluates to true, sets NextState
        app.update(); // state applies

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::AnimateIn,
        );
    }

    #[test]
    fn condition_dispatch_does_not_fire_when_condition_false() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_message::<StateChanged<TestState>>()
            .init_resource::<RoutingTable<TestState>>()
            .add_systems(Update, dispatch_condition_routes::<TestState>);

        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(
                Route::from(TestState::Loading)
                    .to(TestState::AnimateIn)
                    .when(|_world| false),
            )
            .ok();

        app.update();
        app.update();
        app.update();

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );
    }

    #[test]
    fn condition_dispatch_reads_world_in_condition() {
        /// Resource used by the condition closure.
        #[derive(Resource)]
        struct ReadyFlag(bool);

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_message::<StateChanged<TestState>>()
            .init_resource::<RoutingTable<TestState>>()
            .insert_resource(ReadyFlag(false))
            .add_systems(Update, dispatch_condition_routes::<TestState>);

        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(
                Route::from(TestState::Loading)
                    .to(TestState::AnimateIn)
                    .when(|world| world.resource::<ReadyFlag>().0),
            )
            .ok();

        // Flag is false — should not transition
        app.update();
        app.update();
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );

        // Set flag to true — should transition
        app.world_mut().resource_mut::<ReadyFlag>().0 = true;
        app.update();
        app.update();

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::AnimateIn,
        );
    }

    #[test]
    fn condition_dispatch_skips_message_triggered_routes() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_message::<StateChanged<TestState>>()
            .init_resource::<RoutingTable<TestState>>()
            .add_systems(Update, dispatch_condition_routes::<TestState>);

        // Message-triggered route (no .when())
        app.world_mut()
            .resource_mut::<RoutingTable<TestState>>()
            .add(Route::from(TestState::Loading).to(TestState::AnimateIn))
            .ok();

        app.update();
        app.update();
        app.update();

        // Condition dispatch should skip message-triggered routes
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::Loading,
        );
    }

    // --- SubState test ---

    #[test]
    fn dispatch_works_with_substates() {
        #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum Parent {
            #[default]
            Off,
            On,
        }

        #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[source(Parent = Parent::On)]
        enum Child {
            #[default]
            Loading,
            Ready,
        }

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<Parent>()
            .add_sub_state::<Child>()
            .add_message::<ChangeState<Child>>()
            .add_message::<StateChanged<Child>>()
            .init_resource::<RoutingTable<Child>>()
            .add_systems(
                Update,
                dispatch_message_routes::<Child>.run_if(on_message::<ChangeState<Child>>),
            );

        app.world_mut()
            .resource_mut::<RoutingTable<Child>>()
            .add(Route::from(Child::Loading).to(Child::Ready))
            .ok();

        // Activate parent
        app.world_mut()
            .resource_mut::<NextState<Parent>>()
            .set(Parent::On);
        app.update();

        assert_eq!(**app.world().resource::<State<Child>>(), Child::Loading);

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<Child>>>()
            .write(ChangeState::new());
        app.update();
        app.update();

        assert_eq!(**app.world().resource::<State<Child>>(), Child::Ready);
    }
}
