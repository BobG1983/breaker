//! Input actions debug egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{debug::resources::DebugOverlays, input::resources::InputActions};

/// Renders an "Input Actions" egui window listing active actions this frame.
pub fn input_actions_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    actions: Res<InputActions>,
) {
    if !overlays.show_input_actions {
        return;
    }

    bevy_egui::egui::Window::new("Input Actions").show(
        contexts.ctx_mut().expect("primary egui context"),
        |ui| {
            if actions.0.is_empty() {
                ui.label("None");
            } else {
                for action in &actions.0 {
                    ui.label(format!("{action:?}"));
                }
            }
        },
    );
}
