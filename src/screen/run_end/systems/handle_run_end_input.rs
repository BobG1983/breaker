//! System to handle input on the run-end screen.

use bevy::prelude::*;

use crate::{
    input::resources::{GameAction, InputActions},
    shared::GameState,
};

/// Returns to the main menu when the player confirms on the run-end screen.
pub fn handle_run_end_input(
    actions: Res<InputActions>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if actions.active(GameAction::MenuConfirm) {
        next_state.set(GameState::MainMenu);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.init_resource::<InputActions>();
        app.add_systems(Update, handle_run_end_input);
        app
    }

    #[test]
    fn confirm_transitions_to_main_menu() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MenuConfirm);
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("MainMenu"),
            "expected MainMenu, got: {next:?}"
        );
    }

    #[test]
    fn no_confirm_no_transition() {
        let mut app = test_app();
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            !format!("{next:?}").contains("MainMenu"),
            "expected no transition, got: {next:?}"
        );
    }
}
