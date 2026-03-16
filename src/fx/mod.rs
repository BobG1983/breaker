//! Fx domain plugin — cross-cutting visual effects (fade-out, flash, particles).

pub mod components;
mod plugin;
pub mod systems;

pub use components::FadeOut;
pub use plugin::FxPlugin;
