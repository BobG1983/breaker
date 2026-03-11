//! Breaker state telemetry egui window.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    breaker::components::{Breaker, BreakerState, BreakerTilt, BreakerVelocity, BumpState},
    debug::resources::DebugOverlays,
};

/// Renders a "Breaker State" egui window with breaker telemetry.
pub fn breaker_state_ui(
    mut contexts: EguiContexts,
    overlays: Res<DebugOverlays>,
    breaker_query: Query<
        (&BreakerState, &BumpState, &BreakerTilt, &BreakerVelocity),
        With<Breaker>,
    >,
) {
    if !overlays.show_breaker_state {
        return;
    }

    bevy_egui::egui::Window::new("Breaker State").show(
        contexts.ctx_mut().expect("primary egui context"),
        |ui| {
            if let Ok((state, bump, tilt, velocity)) = breaker_query.single() {
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
            } else {
                ui.label("No breaker");
            }
        },
    );
}
