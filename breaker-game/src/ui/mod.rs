//! UI domain plugin — HUD, menus, chip selection screen.

pub(crate) mod components;
pub mod messages;
mod plugin;
pub(crate) mod resources;
pub(crate) mod sets;
pub(crate) mod systems;

pub(crate) use plugin::UiPlugin;
pub(crate) use resources::{TimerUiConfig, TimerUiDefaults};
pub(crate) use sets::UiSystems;
