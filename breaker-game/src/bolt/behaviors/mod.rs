//! Bolt overclock evaluation engine — translates overclock trigger chains into
//! armed triggers and fired effects.

pub(crate) mod active;
pub(crate) mod armed;
pub(crate) mod bridges;
pub(crate) mod effects;
pub(crate) mod evaluate;
pub(crate) mod events;
pub(crate) mod plugin;

pub use active::ActiveOverclocks;
pub(crate) use armed::ArmedTriggers;
pub(crate) use events::OverclockEffectFired;
pub(crate) use plugin::BoltBehaviorsPlugin;
