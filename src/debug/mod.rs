//! Debug domain plugin — `bevy_egui` debug console and overlays.

mod plugin;
pub mod resources;
#[cfg(feature = "dev")]
mod systems;

pub use plugin::DebugPlugin;
