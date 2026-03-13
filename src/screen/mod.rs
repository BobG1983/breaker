//! Screen domain plugin — state registration, transitions, and cleanup.

pub mod loading;
pub mod main_menu;
mod plugin;
pub mod run_end;
pub mod systems;

pub use plugin::ScreenPlugin;
