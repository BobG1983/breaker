//! Telemetry plugin — egui debug panels and bump result tracking.

use bevy::prelude::*;

use super::systems::{
    bolt_info_ui, breaker_state_ui, debug_ui_system, input_actions_ui, track_bump_result,
};
use crate::{debug::resources::DebugOverlays, physics::PhysicsSystems, shared::PlayingState};

/// Registers egui telemetry panels and bump result tracking.
pub struct TelemetryPlugin;

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            (
                debug_ui_system,
                bolt_info_ui,
                breaker_state_ui,
                input_actions_ui,
            )
                .run_if(resource_exists::<DebugOverlays>),
        )
        .add_systems(
            FixedUpdate,
            track_bump_result
                .after(PhysicsSystems::BreakerCollision)
                .run_if(in_state(PlayingState::Active)),
        );
    }
}
