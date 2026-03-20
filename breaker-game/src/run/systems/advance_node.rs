//! System to advance to the next node during a node transition.

use bevy::prelude::*;

use crate::{run::resources::RunState, shared::GameState};

/// Increments the node index and transitions to [`GameState::Playing`].
///
/// Runs on `OnEnter(GameState::NodeTransition)`.
pub(crate) fn advance_node(
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    run_state.node_index += 1;
    run_state.transition_queued = false;
    next_state.set(GameState::Playing);
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
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

    #[test]
    fn sets_next_state_to_playing() {
        let mut app = test_app();
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("Playing"),
            "expected Playing, got: {next:?}"
        );
    }
}
