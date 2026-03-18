//! Main menu components.

use bevy::prelude::*;

/// Marker component on the root main menu UI entity.
#[derive(Component)]
pub(crate) struct MainMenuScreen;

/// Identifies a menu item and its action.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MenuItem {
    /// Start a new run.
    Play,
    /// Open settings (not yet implemented).
    Settings,
    /// Exit the application.
    Quit,
}

/// All menu items in display order.
pub(crate) const MENU_ITEMS: [MenuItem; 3] = [MenuItem::Play, MenuItem::Settings, MenuItem::Quit];
