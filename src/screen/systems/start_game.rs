//! Temporary system to start the game from the main menu.
//!
//! Press Enter or Space to transition from `MainMenu` to `Playing`.
//! This will be replaced by a proper main menu UI in a later phase.

use bevy::prelude::*;

use crate::shared::GameState;

/// Transitions from [`GameState::MainMenu`] to [`GameState::Playing`] on key press.
pub fn start_game_on_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
