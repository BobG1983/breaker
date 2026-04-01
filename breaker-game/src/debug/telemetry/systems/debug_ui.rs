//! Debug UI panel system.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiContexts;

use crate::{
    debug::resources::{DebugOverlays, Overlay},
    shared::GameState,
};

/// Renders the debug UI panel using egui.
pub(crate) fn debug_ui_system(
    mut contexts: EguiContexts,
    mut overlays: ResMut<DebugOverlays>,
    state: Res<State<GameState>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Debug")
        .default_open(true)
        .show(ctx, |ui| {
            ui.heading("Overlays");
            ui.checkbox(overlays.flag_mut(Overlay::Fps), "FPS");
            ui.checkbox(overlays.flag_mut(Overlay::Hitboxes), "Hitboxes");
            ui.checkbox(
                overlays.flag_mut(Overlay::VelocityVectors),
                "Velocity Vectors",
            );
            ui.checkbox(overlays.flag_mut(Overlay::State), "Game State");
            ui.checkbox(overlays.flag_mut(Overlay::BoltInfo), "Bolt Info");
            ui.checkbox(overlays.flag_mut(Overlay::DashState), "Breaker State");
            ui.checkbox(overlays.flag_mut(Overlay::InputActions), "Input Actions");

            ui.separator();

            if overlays.is_active(Overlay::Fps) {
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(bevy::diagnostic::Diagnostic::smoothed)
                {
                    ui.label(format!("FPS: {fps:.1}"));
                } else {
                    ui.label("FPS: --");
                }
            }

            if overlays.is_active(Overlay::State) {
                ui.label(format!("State: {:?}", state.get()));
            }
        });
}
