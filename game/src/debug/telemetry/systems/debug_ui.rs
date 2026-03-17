//! Debug UI panel system.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiContexts;

use crate::{debug::resources::DebugOverlays, shared::GameState};

/// Renders the debug UI panel using egui.
pub fn debug_ui_system(
    mut contexts: EguiContexts,
    mut overlays: ResMut<DebugOverlays>,
    state: Res<State<GameState>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    bevy_egui::egui::Window::new("Debug")
        .default_open(true)
        .show(contexts.ctx_mut().expect("primary egui context"), |ui| {
            ui.heading("Overlays");
            ui.checkbox(&mut overlays.show_fps, "FPS");
            ui.checkbox(&mut overlays.show_hitboxes, "Hitboxes");
            ui.checkbox(&mut overlays.show_velocity_vectors, "Velocity Vectors");
            ui.checkbox(&mut overlays.show_state, "Game State");
            ui.checkbox(&mut overlays.show_bolt_info, "Bolt Info");
            ui.checkbox(&mut overlays.show_breaker_state, "Breaker State");
            ui.checkbox(&mut overlays.show_input_actions, "Input Actions");

            ui.separator();

            if overlays.show_fps {
                if let Some(fps) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(bevy::diagnostic::Diagnostic::smoothed)
                {
                    ui.label(format!("FPS: {fps:.1}"));
                } else {
                    ui.label("FPS: --");
                }
            }

            if overlays.show_state {
                ui.label(format!("State: {:?}", state.get()));
            }
        });
}
