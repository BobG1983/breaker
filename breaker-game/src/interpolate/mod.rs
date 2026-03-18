//! Interpolation domain — smooth visual rendering between fixed-timestep ticks.

pub(crate) mod components;
mod plugin;
pub(crate) mod systems;

pub(crate) use plugin::InterpolatePlugin;
