//! Gizmo overlay for entity hitboxes.

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltRadius},
    breaker::components::{Breaker, BreakerHeight, BreakerWidth},
    cells::components::{Cell, CellHeight, CellWidth},
    debug::resources::{DebugOverlays, Overlay},
};

const BOLT_HITBOX_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const CELL_HITBOX_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const BREAKER_HITBOX_COLOR: Color = Color::srgb(0.0, 0.5, 1.0);

/// Draws gizmo outlines around bolt, cell, and breaker hitboxes.
pub(crate) fn draw_hitboxes(
    overlays: Res<DebugOverlays>,
    mut gizmos: Gizmos,
    bolt_query: Query<(&Transform, &BoltRadius), With<Bolt>>,
    breaker_query: Query<(&Transform, &BreakerWidth, &BreakerHeight), With<Breaker>>,
    cell_query: Query<(&Transform, &CellWidth, &CellHeight), With<Cell>>,
) {
    if !overlays.is_active(Overlay::Hitboxes) {
        return;
    }

    for (transform, radius) in &bolt_query {
        gizmos.circle_2d(
            transform.translation.truncate(),
            radius.0,
            BOLT_HITBOX_COLOR,
        );
    }

    for (transform, cell_w, cell_h) in &cell_query {
        gizmos.rect_2d(
            transform.translation.truncate(),
            Vec2::new(cell_w.0, cell_h.0),
            CELL_HITBOX_COLOR,
        );
    }

    for (transform, breaker_w, breaker_h) in &breaker_query {
        gizmos.rect_2d(
            transform.translation.truncate(),
            Vec2::new(breaker_w.0, breaker_h.0),
            BREAKER_HITBOX_COLOR,
        );
    }
}
