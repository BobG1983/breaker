//! HUD — timer display, side panels, status panel.

pub(crate) mod components;
mod plugin;
pub(crate) mod resources;
pub(crate) mod sets;
pub(crate) mod systems;

pub(crate) use plugin::HudPlugin;
pub(crate) use resources::TimerUiDefaults;
pub(crate) use sets::UiSystems;
