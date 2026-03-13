//! Cleanup system for the main menu.

use bevy::prelude::*;

use crate::screen::main_menu::MainMenuSelection;

/// Removes the selection resource after main menu entity cleanup.
pub fn cleanup_main_menu(mut commands: Commands) {
    commands.remove_resource::<MainMenuSelection>();
}
