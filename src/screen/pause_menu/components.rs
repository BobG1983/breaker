//! Pause menu components.

use bevy::prelude::*;

/// Marker component on the root pause menu UI entity.
#[derive(Component)]
pub struct PauseMenuScreen;

/// Identifies a pause menu item and its action.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseMenuItem {
    /// Resume gameplay.
    Resume,
    /// Quit to main menu.
    Quit,
}

/// All pause menu items in display order.
pub const PAUSE_MENU_ITEMS: [PauseMenuItem; 2] = [PauseMenuItem::Resume, PauseMenuItem::Quit];
