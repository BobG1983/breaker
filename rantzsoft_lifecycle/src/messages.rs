//! Lifecycle messages for state routing.

use std::marker::PhantomData;

use bevy::prelude::*;

/// Request the routing system to transition from the current state.
///
/// Destination-less — the game sends "I'm done with this state" and the
/// [`RoutingTable`](crate::RoutingTable) determines where to go (via
/// static `.to(S)` or dynamic `.to_dynamic(fn)` routes).
///
/// Each concrete instantiation (e.g. `ChangeState<NodeState>`) is a
/// separate message type.
#[derive(Message, Clone)]
pub struct ChangeState<S: States> {
    _phantom: PhantomData<S>,
}

impl<S: States> ChangeState<S> {
    /// Create a new state change request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<S: States> Default for ChangeState<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: States> std::fmt::Debug for ChangeState<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChangeState<{}>", std::any::type_name::<S>())
    }
}

/// Notification sent after a state change has been applied.
///
/// Sent by the routing system after every state transition. Game systems
/// can listen to coordinate audio, analytics, loading indicators, etc.
#[derive(Message, Clone, Debug)]
pub struct StateChanged<S: States> {
    /// The state we transitioned from.
    pub from: S,
    /// The state we transitioned to.
    pub to: S,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum NodeState {
        #[default]
        Loading,
        AnimateIn,
    }

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum MenuState {
        #[default]
        Main,
    }

    #[test]
    fn change_state_for_independent_state_types() {
        let node = ChangeState::<NodeState>::new();
        let menu = ChangeState::<MenuState>::default();
        // Both instantiate without conflict
        assert!(format!("{node:?}").contains("NodeState"));
        assert!(format!("{menu:?}").contains("MenuState"));
    }

    #[test]
    fn state_changed_carries_from_and_to() {
        let msg = StateChanged {
            from: NodeState::Loading,
            to: NodeState::AnimateIn,
        };
        assert_eq!(msg.from, NodeState::Loading);
        assert_eq!(msg.to, NodeState::AnimateIn);
    }

    #[test]
    fn independent_types_register_as_separate_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChangeState<NodeState>>()
            .add_message::<ChangeState<MenuState>>()
            .add_message::<StateChanged<NodeState>>()
            .add_message::<StateChanged<MenuState>>();

        // Write to NodeState, MenuState stays empty
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<NodeState>>>()
            .write(ChangeState::new());
        app.update();

        let menu_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<ChangeState<MenuState>>>();
        assert_eq!(
            menu_msgs.iter_current_update_messages().count(),
            0,
            "ChangeState<MenuState> should have no messages"
        );
    }
}
