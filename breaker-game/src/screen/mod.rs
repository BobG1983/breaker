//! Screen domain plugin — state registration, transitions, and cleanup.

pub(crate) mod chip_select;
pub(crate) mod loading;
pub(crate) mod main_menu;
pub(crate) mod pause_menu;
mod plugin;
pub(crate) mod run_end;
pub(crate) mod run_setup;
pub(crate) mod systems;

pub(crate) use plugin::ScreenPlugin;
