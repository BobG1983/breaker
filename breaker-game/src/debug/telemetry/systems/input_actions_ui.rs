//! Input actions debug egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    debug::resources::{DebugOverlays, Overlay},
    input::resources::InputActions,
};

/// Renders an "Input Actions" egui window listing active actions this frame.
pub(crate) fn input_actions_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    actions: Res<InputActions>,
) {
    if !overlays.is_active(Overlay::InputActions) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Input Actions").show(ctx, |ui| {
        if actions.0.is_empty() {
            ui.label("None");
        } else {
            for action in &actions.0 {
                ui.label(format!("{action:?}"));
            }
        }
    });
}
