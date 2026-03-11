//! Loading state transition system.

use bevy::prelude::*;

use crate::shared::GameState;

/// Immediately transitions from [`GameState::Loading`] to [`GameState::MainMenu`].
///
/// In later phases this will wait for asset loading to complete.
pub fn finish_loading(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}
