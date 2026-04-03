//! Pause menu sub-domain — overlay during gameplay.

mod components;
mod plugin;
pub(crate) mod resources;
mod systems;

pub(crate) use components::PauseMenuScreen;
pub(crate) use plugin::PauseMenuPlugin;
