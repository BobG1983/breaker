//! Bolt telemetry egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    debug::resources::{DebugOverlays, Overlay},
    prelude::*,
};

/// Renders a "Bolt Info" egui window with bolt telemetry.
pub(crate) fn bolt_info_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    bolt_query: Query<(&Transform, &Velocity2D), With<Bolt>>,
) {
    if !overlays.is_active(Overlay::BoltInfo) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Bolt Info").show(ctx, |ui| {
        let mut found = false;
        for (i, (transform, velocity)) in bolt_query.iter().enumerate() {
            found = true;
            if i > 0 {
                ui.separator();
            }
            let pos = transform.translation.truncate();
            let speed = velocity.speed();
            let angle_deg = velocity.0.y.atan2(velocity.0.x).to_degrees();

            ui.label(format!("Bolt {i}"));
            ui.label(format!("  pos: ({:.1}, {:.1})", pos.x, pos.y));
            ui.label(format!("  speed: {speed:.1}"));
            ui.label(format!("  angle: {angle_deg:.1} deg"));
            ui.label(format!("  vel: ({:.1}, {:.1})", velocity.0.x, velocity.0.y));
        }
        if !found {
            ui.label("No bolt");
        }
    });
}
