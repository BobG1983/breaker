//! Gizmo overlay for velocity vectors.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    debug::resources::{DebugOverlays, Overlay},
};

const VELOCITY_ARROW_SCALE: f32 = 0.25;
const BOLT_ARROW_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const BREAKER_ARROW_COLOR: Color = Color::srgb(0.0, 0.5, 1.0);

/// Draws gizmo arrows showing bolt and breaker velocity vectors.
pub(crate) fn draw_velocity_vectors(
    overlays: Res<DebugOverlays>,
    mut gizmos: Gizmos,
    bolt_query: Query<(&Transform, &Velocity2D), With<Bolt>>,
    breaker_query: Query<(&Transform, &Velocity2D), With<Breaker>>,
) {
    if !overlays.is_active(Overlay::VelocityVectors) {
        return;
    }

    for (transform, velocity) in &bolt_query {
        let start = transform.translation.truncate();
        let end = start + velocity.0 * VELOCITY_ARROW_SCALE;
        gizmos.arrow_2d(start, end, BOLT_ARROW_COLOR);
    }

    for (transform, velocity) in &breaker_query {
        let start = transform.translation.truncate();
        let end = start + Vec2::new(velocity.0.x * VELOCITY_ARROW_SCALE, 0.0);
        gizmos.arrow_2d(start, end, BREAKER_ARROW_COLOR);
    }
}
