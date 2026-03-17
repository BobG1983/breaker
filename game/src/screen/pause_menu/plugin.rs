//! Pause menu plugin registration.

use bevy::prelude::*;

use super::{
    PauseMenuScreen,
    systems::{handle_pause_input, spawn_pause_menu, toggle_pause},
};
use crate::shared::{GameState, PlayingState};

/// Plugin for the pause menu overlay.
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_pause.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(PlayingState::Paused), spawn_pause_menu)
            .add_systems(
                Update,
                handle_pause_input.run_if(in_state(PlayingState::Paused)),
            )
            .add_systems(
                OnExit(PlayingState::Paused),
                crate::screen::systems::cleanup_entities::<PauseMenuScreen>,
            );
    }
}
