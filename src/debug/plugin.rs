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

            app.add_plugins(EguiPlugin::default());
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
            app.init_resource::<DebugOverlays>();
            app.add_systems(
                bevy_egui::EguiPrimaryContextPass,
                super::systems::debug_ui_system.run_if(resource_exists::<DebugOverlays>),
            );
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
