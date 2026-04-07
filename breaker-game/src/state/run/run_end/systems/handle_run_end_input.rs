//! System to handle input on the run-end screen.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

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
    mut writer: MessageWriter<ChangeState<RunEndState>>,
) {
    if actions.active(GameAction::MenuConfirm) {
        writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::message::Messages, state::app::StatesPlugin};
    use rantzsoft_stateflow::ChangeState;

    use super::*;
    use crate::state::types::{AppState, GameState, RunState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<RunEndState>()
            .add_message::<ChangeState<RunEndState>>()
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
            .resource_mut::<NextState<RunState>>()
            .set(RunState::RunEnd);
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

        let msgs = app.world().resource::<Messages<ChangeState<RunEndState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<RunEndState> message"
        );
    }

    #[test]
    fn no_confirm_no_transition() {
        let mut app = test_app();
        app.update();

        let msgs = app.world().resource::<Messages<ChangeState<RunEndState>>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "expected no ChangeState message"
        );
    }
}
