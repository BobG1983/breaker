//! System to complete the transition-out phase by entering chip selection.

use bevy::prelude::*;

use crate::shared::GameState;

/// Immediately transitions from [`GameState::TransitionOut`] to [`GameState::ChipSelect`].
///
/// Runs on `OnEnter(GameState::TransitionOut)`. Will be replaced by timed
/// transition animation in a later commit.
pub(crate) fn complete_transition_out(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::ChipSelect);
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .add_systems(Update, complete_transition_out);
        app
    }

    #[test]
    fn sets_next_state_to_chip_select() {
        let mut app = test_app();
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("ChipSelect"),
            "expected ChipSelect, got: {next:?}"
        );
    }
}
