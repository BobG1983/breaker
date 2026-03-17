//! Debug domain plugin — `bevy_egui` debug console and overlays.

#[cfg(feature = "dev")]
mod hot_reload;
#[cfg(feature = "dev")]
mod overlays;
mod plugin;
pub mod resources;
#[cfg(feature = "dev")]
mod telemetry;

pub use plugin::DebugPlugin;
