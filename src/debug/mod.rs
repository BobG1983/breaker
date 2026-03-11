//! Debug domain plugin — `bevy_egui` debug console and overlays.

mod plugin;
mod resources;
#[cfg(feature = "dev")]
mod systems;

pub use plugin::DebugPlugin;
pub use resources::DebugOverlays;
