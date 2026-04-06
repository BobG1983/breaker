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
    transition::{
        orchestration::begin_transition,
        resources::ActiveTransition,
        types::{TransitionKind, TransitionType},
    },
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

    if count == 0 {
        return;
    }

    // Defer if transition is active — re-queue for next frame
    if world.contains_resource::<ActiveTransition>() {
        world
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<S>>>()
            .write(ChangeState::new());
        return;
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

        // Resolve transition
        let transition = resolve_transition(&route.transition, world);

        // Execute route
        execute_route::<S>(world, current, destination, transition);
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

    // Defer if transition is active — condition will re-evaluate next frame
    if world.contains_resource::<ActiveTransition>() {
        return;
    }

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

        // Resolve transition
        let transition = resolve_transition(&route.transition, world);

        // Execute route
        execute_route::<S>(world, current, destination, transition);
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

/// Resolve the transition from a route's `TransitionKind`.
fn resolve_transition(transition: &TransitionKind, world: &World) -> Option<TransitionType> {
    match transition {
        TransitionKind::None => None,
        TransitionKind::Static(t) => Some(t.clone()),
        TransitionKind::Dynamic(f) => Some(f(world)),
    }
}

/// Execute a resolved route — either direct state change or begin transition.
fn execute_route<S: FreelyMutableState + Copy>(
    world: &mut World,
    from: S,
    to: S,
    transition: Option<TransitionType>,
) {
    match transition {
        None => {
            // Direct state change (no transition)
            world.resource_mut::<NextState<S>>().set(to);
            world
                .resource_mut::<bevy::ecs::message::Messages<StateChanged<S>>>()
                .write(StateChanged { from, to });
        }
        Some(t) => {
            // Begin transition orchestration
            begin_transition::<S>(world, from, to, t);
        }
    }
}
