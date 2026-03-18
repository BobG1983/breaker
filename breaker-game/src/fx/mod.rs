//! Fx domain plugin — cross-cutting visual effects (fade-out, flash, particles).

pub(crate) mod components;
mod plugin;
pub(crate) mod systems;

pub(crate) use components::FadeOut;
pub(crate) use plugin::FxPlugin;
