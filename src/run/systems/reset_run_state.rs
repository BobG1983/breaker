//! System to reset run state at the start of a new run.

use bevy::prelude::*;

use crate::run::resources::RunState;

/// Resets [`RunState`] to defaults when leaving the main menu (starting a run).
pub fn reset_run_state(mut run_state: ResMut<RunState>) {
    *run_state = RunState::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::RunOutcome;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(RunState {
            node_index: 5,
            outcome: RunOutcome::Won,
            ..default()
        });
        app.add_systems(Update, reset_run_state);
        app
    }

    #[test]
    fn resets_to_defaults() {
        let mut app = test_app();
        app.update();

        let state = app.world().resource::<RunState>();
        assert_eq!(state.node_index, 0);
        assert_eq!(state.outcome, RunOutcome::InProgress);
    }
}
