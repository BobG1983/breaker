//! Breaker state telemetry egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    breaker::queries::BreakerTelemetryData,
    debug::resources::{DebugOverlays, LastBumpResult, Overlay},
    prelude::*,
};

/// Renders a "Breaker State" egui window with breaker telemetry.
pub(crate) fn breaker_state_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    last_bump: Res<LastBumpResult>,
    breaker_query: Query<BreakerTelemetryData, With<Breaker>>,
) {
    if !overlays.is_active(Overlay::DashState) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    bevy_egui::egui::Window::new("Breaker State").show(ctx, |ui| {
        if let Ok(data) = breaker_query.single() {
            ui.label(format!("State: {:?}", data.state));
            ui.label(format!("Velocity X: {:.1}", data.velocity.0.x));
            ui.label(format!(
                "Tilt: {:.3} rad ({:.1} deg)",
                data.tilt.angle,
                data.tilt.angle.to_degrees()
            ));
            ui.separator();
            ui.label(format!("Bump active: {}", data.bump.active));
            ui.label(format!("Bump timer: {:.3}", data.bump.timer));
            ui.label(format!("Bump cooldown: {:.3}", data.bump.cooldown));
            ui.label(format!("Post-hit timer: {:.3}", data.bump.post_hit_timer));
            let can_bump = data.bump.cooldown <= 0.0 && !data.bump.active;
            ui.label(format!("Can bump: {can_bump}"));
            if !last_bump.0.is_empty() {
                ui.label(format!("Last result: {}", last_bump.0));
            }
            ui.separator();
            ui.label(format!("Perfect window: {:.3}", data.perfect_window.0));
            ui.label(format!("Early window: {:.3}", data.early_window.0));
            ui.label(format!("Late window: {:.3}", data.late_window.0));
        } else {
            ui.label("No breaker");
        }
    });
}
