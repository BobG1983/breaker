//! System to handle input on the run-end screen.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::{input::resources::GameAction, prelude::*};

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
    use bevy::ecs::message::Messages;
    use rantzsoft_stateflow::ChangeState;

    use super::*;

    fn test_app() -> App {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_message::<ChangeState<RunEndState>>()
            .with_resource::<InputActions>()
            .with_system(Update, handle_run_end_input)
            .build();
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
