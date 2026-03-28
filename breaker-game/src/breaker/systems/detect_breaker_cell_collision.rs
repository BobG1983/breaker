//! Breaker-cell collision detection.
//!
//! Detects when the breaker entity overlaps a cell entity and sends
//! [`BreakerImpactCell`] messages. Used by effect triggers to fire
//! `Impact(Cell)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{Breaker, BreakerHeight, BreakerWidth},
        messages::BreakerImpactCell,
    },
    cells::components::Cell,
    shared::EntityScale,
};

/// Breaker query data for cell collision detection.
type BreakerCellCollisionQuery = (
    Entity,
    &'static Position2D,
    &'static BreakerWidth,
    &'static BreakerHeight,
    Option<&'static EntityScale>,
);

/// Detects breaker-cell collisions via AABB overlap.
///
/// For each breaker, checks all cell entities for overlap. Sends
/// [`BreakerImpactCell`] for each detected collision. Currently a
/// placeholder: future moving-cell mechanics will make this active.
pub(crate) fn detect_breaker_cell_collision(
    breaker_query: Query<BreakerCellCollisionQuery, With<Breaker>>,
    cell_query: Query<(Entity, &Position2D, &Aabb2D), With<Cell>>,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    let Ok((breaker_entity, breaker_pos, breaker_w, breaker_h, breaker_scale)) =
        breaker_query.single()
    else {
        return;
    };

    let scale = breaker_scale.map_or(1.0, |s| s.0);
    let half_w = breaker_w.half_width() * scale;
    let half_h = breaker_h.half_height() * scale;

    for (cell_entity, cell_pos, cell_aabb) in &cell_query {
        let cell_half = cell_aabb.half_extents;

        // Simple AABB overlap test
        let dx = (breaker_pos.0.x - cell_pos.0.x).abs();
        let dy = (breaker_pos.0.y - cell_pos.0.y).abs();

        if dx < half_w + cell_half.x && dy < half_h + cell_half.y {
            writer.write(BreakerImpactCell {
                breaker: breaker_entity,
                cell: cell_entity,
            });
        }
    }
}
