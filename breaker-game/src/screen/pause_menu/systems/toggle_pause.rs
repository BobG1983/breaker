//! System to toggle pause state with Escape key.

use bevy::prelude::*;

use crate::{
    input::resources::{GameAction, InputActions},
    shared::PlayingState,
};

/// Toggles between [`PlayingState::Active`] and [`PlayingState::Paused`] on `TogglePause`.
///
/// Reads [`InputActions`] for the [`GameAction::TogglePause`] action.
pub(crate) fn toggle_pause(
    actions: Res<InputActions>,
    current_state: Res<State<PlayingState>>,
    mut next_state: ResMut<NextState<PlayingState>>,
) {
    if !actions.active(GameAction::TogglePause) {
        return;
    }

    match current_state.get() {
        PlayingState::Active => next_state.set(PlayingState::Paused),
        PlayingState::Paused => next_state.set(PlayingState::Active),
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{input::resources::*, shared::GameState};

    fn test_app(initial_playing_state: PlayingState) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<InputActions>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();

        // Transition to Playing state so PlayingState is active
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // Set initial PlayingState
        if initial_playing_state != PlayingState::Active {
            app.world_mut()
                .resource_mut::<NextState<PlayingState>>()
                .set(initial_playing_state);
            app.update();
        }

        app.add_systems(Update, toggle_pause);
        app
    }

    fn inject_toggle_pause(app: &mut App) {
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::TogglePause);
        app.update();
    }

    #[test]
    fn escape_toggles_active_to_paused() {
        let mut app = test_app(PlayingState::Active);
        inject_toggle_pause(&mut app);

        let next = app.world().resource::<NextState<PlayingState>>();
        assert!(
            format!("{next:?}").contains("Paused"),
            "expected Paused, got: {next:?}"
        );
    }

    #[test]
    fn escape_toggles_paused_to_active() {
        let mut app = test_app(PlayingState::Paused);
        inject_toggle_pause(&mut app);

        let next = app.world().resource::<NextState<PlayingState>>();
        assert!(
            format!("{next:?}").contains("Active"),
            "expected Active, got: {next:?}"
        );
    }

    #[test]
    fn no_escape_no_change() {
        let mut app = test_app(PlayingState::Active);
        app.update();

        let next = app.world().resource::<NextState<PlayingState>>();
        assert!(
            !format!("{next:?}").contains("Paused"),
            "expected no state change, got: {next:?}"
        );
    }
}
