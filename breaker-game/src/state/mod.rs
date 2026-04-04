//! State domain — state lifecycle, routing, transitions, screens, HUD.

pub(crate) mod app;
pub(crate) mod cleanup;
pub(crate) mod menu;
pub(crate) mod pause;
mod plugin;
pub mod run;
// transition module parked — replaced by lifecycle crate in Wave 5
// pub(crate) mod transition;
pub mod types;

pub(crate) use plugin::StatePlugin;
