//! Screen domain components.

use bevy::prelude::*;

/// Marker component on the root main menu UI entity.
#[derive(Component)]
pub struct MainMenuScreen;

/// Identifies a menu item and its action.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuItem {
    /// Start a new run.
    Play,
    /// Open settings (not yet implemented).
    Settings,
    /// Exit the application.
    Quit,
}

/// All menu items in display order.
pub const MENU_ITEMS: [MenuItem; 3] = [MenuItem::Play, MenuItem::Settings, MenuItem::Quit];

/// Marker component for loading screen entities.
#[derive(Component)]
pub struct LoadingScreen;

/// Marker for the loading progress bar inner fill.
#[derive(Component)]
pub struct LoadingBarFill;

/// Marker for the loading progress text.
#[derive(Component)]
pub struct LoadingProgressText;
