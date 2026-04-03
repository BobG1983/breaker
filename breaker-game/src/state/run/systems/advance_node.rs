//! System to advance to the next node during a node transition.

use bevy::prelude::*;

use crate::state::run::resources::RunState;

/// Increments the node index and resets the transition flag.
///
/// Runs on `OnEnter(RunPhase::Node)` — called when entering a new node
/// after chip select.
pub(crate) fn advance_node(mut run_state: ResMut<RunState>) {
    run_state.node_index += 1;
    run_state.transition_queued = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RunState {
                node_index: 0,
                transition_queued: true,
                ..default()
            })
            .add_systems(Update, advance_node);
        app
    }

    #[test]
    fn increments_node_index() {
        let mut app = test_app();
        app.update();

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.node_index, 1);
    }

    #[test]
    fn resets_transition_queued() {
        let mut app = test_app();
        app.update();

        let run_state = app.world().resource::<RunState>();
        assert!(
            !run_state.transition_queued,
            "transition_queued should be reset for the next node"
        );
    }
}
