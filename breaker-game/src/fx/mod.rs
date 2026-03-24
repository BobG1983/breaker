//! Fx domain plugin — cross-cutting visual effects (fade-out, flash, particles).

pub(crate) mod components;
mod plugin;
pub(crate) mod systems;
pub(crate) mod transition;

pub(crate) use components::{FadeOut, PunchScale};
pub(crate) use plugin::FxPlugin;
pub(crate) use transition::{TransitionConfig, TransitionDefaults, TransitionOverlay};
