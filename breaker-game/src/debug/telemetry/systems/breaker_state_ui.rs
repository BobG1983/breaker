//! Breaker state telemetry egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    breaker::{components::Breaker, queries::BumpTelemetryQuery},
    debug::resources::{DebugOverlays, LastBumpResult, Overlay},
};

/// Renders a "Breaker State" egui window with breaker telemetry.
pub(crate) fn breaker_state_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    last_bump: Res<LastBumpResult>,
    breaker_query: Query<BumpTelemetryQuery, With<Breaker>>,
) {
    if !overlays.is_active(Overlay::BreakerState) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Breaker State").show(ctx, |ui| {
        if let Ok((state, bump, tilt, velocity, perfect_w, early_w, late_w)) =
            breaker_query.single()
        {
            ui.label(format!("State: {state:?}"));
            ui.label(format!("Velocity X: {:.1}", velocity.x));
            ui.label(format!(
                "Tilt: {:.3} rad ({:.1} deg)",
                tilt.angle,
                tilt.angle.to_degrees()
            ));
            ui.separator();
            ui.label(format!("Bump active: {}", bump.active));
            ui.label(format!("Bump timer: {:.3}", bump.timer));
            ui.label(format!("Bump cooldown: {:.3}", bump.cooldown));
            ui.label(format!("Post-hit timer: {:.3}", bump.post_hit_timer));
            let can_bump = bump.cooldown <= 0.0 && !bump.active;
            ui.label(format!("Can bump: {can_bump}"));
            if !last_bump.0.is_empty() {
                ui.label(format!("Last result: {}", last_bump.0));
            }
            ui.separator();
            ui.label(format!("Perfect window: {:.3}", perfect_w.0));
            ui.label(format!("Early window: {:.3}", early_w.0));
            ui.label(format!("Late window: {:.3}", late_w.0));
        } else {
            ui.label("No breaker");
        }
    });
}
