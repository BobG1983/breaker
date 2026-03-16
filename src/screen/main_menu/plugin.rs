//! Main menu plugin registration.

use bevy::prelude::*;

use super::{
    MainMenuScreen,
    systems::{handle_main_menu_input, spawn_main_menu, update_menu_colors},
};
use crate::shared::GameState;

/// Plugin for the main menu screen.
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                (handle_main_menu_input, update_menu_colors)
                    .chain()
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::MainMenu),
                crate::screen::systems::cleanup_entities::<MainMenuScreen>,
            );
    }
}
