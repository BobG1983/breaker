//! Overlay plugin — gizmo drawing for hitboxes and velocity vectors.

use bevy::prelude::*;

use super::systems::{draw_hitboxes, draw_velocity_vectors};
use crate::debug::resources::DebugOverlays;

/// Registers gizmo overlay systems.
pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_hitboxes, draw_velocity_vectors).run_if(resource_exists::<DebugOverlays>),
        );
    }
}
