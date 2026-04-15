//! Main menu plugin registration.

use bevy::prelude::*;

use super::{
    MainMenuScreen,
    systems::{handle_main_menu_input, spawn_main_menu, update_menu_colors},
};
use crate::{prelude::*, state::cleanup::cleanup_entities};

/// Plugin for the main menu screen.
pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Main), spawn_main_menu)
            .add_systems(
                Update,
                (handle_main_menu_input, update_menu_colors)
                    .chain()
                    .run_if(in_state(MenuState::Main)),
            )
            .add_systems(OnExit(MenuState::Main), cleanup_entities::<MainMenuScreen>);
    }
}
