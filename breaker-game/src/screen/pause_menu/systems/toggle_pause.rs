//! System to toggle pause state with Escape key.

use bevy::prelude::*;

use crate::shared::PlayingState;

/// Toggles between [`PlayingState::Active`] and [`PlayingState::Paused`] on Escape.
///
/// Reads `ButtonInput<KeyCode>` directly for the Escape key.
pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<PlayingState>>,
    mut next_state: ResMut<NextState<PlayingState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
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
    use crate::shared::GameState;

    fn test_app(initial_playing_state: PlayingState) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
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

    fn press_escape(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
    }

    #[test]
    fn escape_toggles_active_to_paused() {
        let mut app = test_app(PlayingState::Active);
        press_escape(&mut app);

        let next = app.world().resource::<NextState<PlayingState>>();
        assert!(
            format!("{next:?}").contains("Paused"),
            "expected Paused, got: {next:?}"
        );
    }

    #[test]
    fn escape_toggles_paused_to_active() {
        let mut app = test_app(PlayingState::Paused);
        press_escape(&mut app);

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
