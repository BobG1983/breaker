//! Screen domain plugin — state registration, transitions, and cleanup.

pub mod chip_select;
pub mod loading;
pub mod main_menu;
pub mod pause_menu;
mod plugin;
pub mod run_end;
pub mod run_setup;
pub mod systems;

pub use plugin::ScreenPlugin;
