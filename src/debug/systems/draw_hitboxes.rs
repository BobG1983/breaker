//! Gizmo overlay for entity hitboxes.

use bevy::prelude::*;

use crate::{
    bolt::{components::Bolt, resources::BoltConfig},
    breaker::{components::Breaker, resources::BreakerConfig},
    cells::{components::Cell, resources::CellConfig},
    debug::resources::DebugOverlays,
};

const BOLT_HITBOX_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const CELL_HITBOX_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BREAKER_HITBOX_COLOR: Color = Color::srgb(0.0, 0.5, 1.0);

/// Draws gizmo outlines around bolt, cell, and breaker hitboxes.
#[allow(clippy::too_many_arguments)]
pub fn draw_hitboxes(
    overlays: Res<DebugOverlays>,
    mut gizmos: Gizmos,
    bolt_config: Res<BoltConfig>,
    breaker_config: Res<BreakerConfig>,
    cell_config: Res<CellConfig>,
    bolt_query: Query<&Transform, With<Bolt>>,
    breaker_query: Query<&Transform, With<Breaker>>,
    cell_query: Query<&Transform, With<Cell>>,
) {
    if !overlays.show_hitboxes {
        return;
    }

    for transform in &bolt_query {
        gizmos.circle_2d(
            transform.translation.truncate(),
            bolt_config.radius,
            BOLT_HITBOX_COLOR,
        );
    }

    for transform in &cell_query {
        gizmos.rect_2d(
            transform.translation.truncate(),
            Vec2::new(cell_config.half_width * 2.0, cell_config.half_height * 2.0),
            CELL_HITBOX_COLOR,
        );
    }

    for transform in &breaker_query {
        gizmos.rect_2d(
            transform.translation.truncate(),
            Vec2::new(
                breaker_config.half_width * 2.0,
                breaker_config.half_height * 2.0,
            ),
            BREAKER_HITBOX_COLOR,
        );
    }
}
