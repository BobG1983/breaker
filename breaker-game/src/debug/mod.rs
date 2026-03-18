//! Debug domain plugin — `bevy_egui` debug console and overlays.

#[cfg(feature = "dev")]
mod hot_reload;
#[cfg(feature = "dev")]
mod overlays;
mod plugin;
#[cfg(feature = "dev")]
pub(crate) mod recording;
pub(crate) mod resources;
#[cfg(feature = "dev")]
mod telemetry;

pub(crate) use plugin::DebugPlugin;
