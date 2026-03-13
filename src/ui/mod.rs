//! UI domain plugin — HUD, menus, upgrade selection screen.

pub mod components;
pub mod messages;
mod plugin;
pub mod resources;
pub mod systems;

pub use plugin::UiPlugin;
pub use resources::{TimerUiConfig, TimerUiDefaults};
