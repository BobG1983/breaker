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
    pub to:   S,
}

/// Sent at the start of a transition lifecycle.
///
/// For Out/OutIn transitions, sent before the Out effect begins.
/// For In/OneShot transitions, sent before the state change and effect.
/// Game systems can listen to prepare for the transition (e.g., fade audio).
#[derive(Message, Clone, Debug)]
pub struct TransitionStart<S: States> {
    /// The state we are transitioning from.
    pub from: S,
    /// The state we are transitioning to.
    pub to:   S,
}

/// Sent at the end of a transition lifecycle.
///
/// Sent after the final cleanup — state has changed, effects have completed,
/// and `ActiveTransition` has been removed.
#[derive(Message, Clone, Debug)]
pub struct TransitionEnd<S: States> {
    /// The state we transitioned from.
    pub from: S,
    /// The state we transitioned to.
    pub to:   S,
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
            to:   NodeState::AnimateIn,
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

    // --- Section E: TransitionStart<S> and TransitionEnd<S> ---

    #[test]
    fn transition_start_carries_from_and_to() {
        let msg = TransitionStart {
            from: NodeState::Loading,
            to:   NodeState::AnimateIn,
        };
        assert_eq!(msg.from, NodeState::Loading);
        assert_eq!(msg.to, NodeState::AnimateIn);
    }

    #[test]
    fn transition_start_different_state_types_are_independent() {
        let node_msg = TransitionStart {
            from: NodeState::Loading,
            to:   NodeState::AnimateIn,
        };
        let menu_msg = TransitionStart {
            from: MenuState::Main,
            to:   MenuState::Main,
        };
        // Both compile and instantiate independently — verify fields accessible
        let _ = &node_msg;
        let _ = &menu_msg;
    }

    #[test]
    fn transition_end_carries_from_and_to() {
        let msg = TransitionEnd {
            from: NodeState::Loading,
            to:   NodeState::AnimateIn,
        };
        assert_eq!(msg.from, NodeState::Loading);
        assert_eq!(msg.to, NodeState::AnimateIn);
    }

    #[test]
    fn transition_end_different_state_types_are_independent() {
        let node_msg = TransitionEnd {
            from: NodeState::Loading,
            to:   NodeState::AnimateIn,
        };
        let menu_msg = TransitionEnd {
            from: MenuState::Main,
            to:   MenuState::Main,
        };
        // Both compile and instantiate independently — verify fields accessible
        let _ = &node_msg;
        let _ = &menu_msg;
    }

    #[test]
    fn transition_start_and_end_register_as_separate_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionStart<NodeState>>()
            .add_message::<TransitionEnd<NodeState>>();

        // Write TransitionStart, TransitionEnd should be empty
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionStart<NodeState>>>()
            .write(TransitionStart {
                from: NodeState::Loading,
                to:   NodeState::AnimateIn,
            });
        app.update();

        let end_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionEnd<NodeState>>>();
        assert_eq!(
            end_msgs.iter_current_update_messages().count(),
            0,
            "TransitionEnd should have no messages when only TransitionStart was written"
        );
    }

    #[test]
    fn transition_start_and_end_coexist_in_same_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionStart<NodeState>>()
            .add_message::<TransitionEnd<NodeState>>();

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionStart<NodeState>>>()
            .write(TransitionStart {
                from: NodeState::Loading,
                to:   NodeState::AnimateIn,
            });
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionEnd<NodeState>>>()
            .write(TransitionEnd {
                from: NodeState::Loading,
                to:   NodeState::AnimateIn,
            });
        app.update();

        let start_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionStart<NodeState>>>();
        assert_eq!(start_msgs.iter_current_update_messages().count(), 1);

        let end_msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionEnd<NodeState>>>();
        assert_eq!(end_msgs.iter_current_update_messages().count(), 1);
    }
}
