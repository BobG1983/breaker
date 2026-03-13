//! Main menu sub-domain — menu UI, navigation, and selection.

mod components;
mod plugin;
mod resources;
mod systems;

pub use components::{MENU_ITEMS, MainMenuScreen, MenuItem};
pub use plugin::MainMenuPlugin;
pub use resources::{MainMenuConfig, MainMenuDefaults, MainMenuSelection};
