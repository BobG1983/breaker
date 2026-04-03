//! System to handle input on the run-end screen.

use bevy::prelude::*;

use crate::{
    input::resources::{GameAction, InputActions},
    state::types::RunEndState,
};

/// Returns to the main menu when the player confirms on the run-end screen.
///
/// Sets [`RunEndState::AnimateOut`], which triggers teardown routing back
/// through the parent hierarchy to the menu.
pub(crate) fn handle_run_end_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<RunEndState>>,
) {
    if actions.active(GameAction::MenuConfirm) {
        next_state.set(RunEndState::AnimateOut);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::state::types::{AppState, GameState, RunPhase};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<RunEndState>()
            .init_resource::<InputActions>()
            .add_systems(Update, handle_run_end_input);
        // Navigate to RunEndState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunPhase>>()
            .set(RunPhase::RunEnd);
        app.update();
        app
    }

    #[test]
    fn confirm_transitions_to_animate_out() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MenuConfirm);
        app.update();

        let next = app.world().resource::<NextState<RunEndState>>();
        assert!(
            format!("{next:?}").contains("AnimateOut"),
            "expected AnimateOut, got: {next:?}"
        );
    }

    #[test]
    fn no_confirm_no_transition() {
        let mut app = test_app();
        app.update();

        let next = app.world().resource::<NextState<RunEndState>>();
        assert!(
            !format!("{next:?}").contains("AnimateOut"),
            "expected no transition, got: {next:?}"
        );
    }
}
