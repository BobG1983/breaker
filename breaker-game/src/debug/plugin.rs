//! Debug plugin registration.

use bevy::prelude::*;

use super::resources::DebugOverlays;

/// Plugin for debug tooling.
///
/// Provides an in-game debug panel with overlay toggles, state inspection,
/// and FPS display. Only active when the `dev` feature is enabled.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(feature = "dev")]
        {
            use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
            use bevy_egui::EguiPlugin;

            use super::{
                hot_reload::plugin::HotReloadPlugin, overlays::plugin::OverlaysPlugin,
                resources::LastBumpResult, telemetry::plugin::TelemetryPlugin,
            };

            app.add_plugins(EguiPlugin::default())
                .add_plugins(FrameTimeDiagnosticsPlugin::default())
                .add_plugins(OverlaysPlugin)
                .add_plugins(TelemetryPlugin)
                .add_plugins(HotReloadPlugin)
                .init_resource::<DebugOverlays>()
                .init_resource::<LastBumpResult>();
        }
    }
}

/// Tests the non-dev path (no-op build). The dev path requires a render
/// context and is tested via `cargo dev`.
#[cfg(all(test, not(feature = "dev")))]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds_headless() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(DebugPlugin)
            .update();
    }
}
