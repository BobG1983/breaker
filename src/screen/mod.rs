//! Screen domain plugin — state registration, transitions, and cleanup.

pub mod loading;
pub mod main_menu;
pub mod pause_menu;
mod plugin;
pub mod run_end;
pub mod run_setup;
pub mod systems;
pub mod upgrade_select;

pub use plugin::ScreenPlugin;
