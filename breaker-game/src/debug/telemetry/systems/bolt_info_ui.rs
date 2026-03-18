//! Bolt telemetry egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    bolt::components::{Bolt, BoltVelocity},
    debug::resources::{DebugOverlays, Overlay},
};

/// Renders a "Bolt Info" egui window with bolt telemetry.
pub(crate) fn bolt_info_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    bolt_query: Query<(&Transform, &BoltVelocity), With<Bolt>>,
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
            let angle_deg = velocity.value.y.atan2(velocity.value.x).to_degrees();

            ui.label(format!("Bolt {i}"));
            ui.label(format!("  pos: ({:.1}, {:.1})", pos.x, pos.y));
            ui.label(format!("  speed: {speed:.1}"));
            ui.label(format!("  angle: {angle_deg:.1} deg"));
            ui.label(format!(
                "  vel: ({:.1}, {:.1})",
                velocity.value.x, velocity.value.y
            ));
        }
        if !found {
            ui.label("No bolt");
        }
    });
}
