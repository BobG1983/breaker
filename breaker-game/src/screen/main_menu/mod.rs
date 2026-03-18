//! Main menu sub-domain — menu UI, navigation, and selection.

mod components;
mod plugin;
mod resources;
mod systems;

pub(crate) use components::{MENU_ITEMS, MainMenuScreen, MenuItem};
pub(crate) use plugin::MainMenuPlugin;
pub(crate) use resources::{MainMenuConfig, MainMenuDefaults, MainMenuSelection};
