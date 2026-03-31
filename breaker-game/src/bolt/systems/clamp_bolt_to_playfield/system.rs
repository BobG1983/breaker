//! Safety clamp -- catches bolts that escape through wall corner overlaps.

use bevy::prelude::*;

use crate::{
    bolt::{
        filters::ActiveFilter,
        queries::{BoltCollisionData, apply_velocity_formula},
    },
    shared::PlayfieldConfig,
};

/// Sub-pixel inset applied when clamping bolt position to playfield walls.
///
/// Keeps the bolt just inside the boundary to prevent floating-point edge overlap
/// on the next frame.
const BOUNDARY_INSET: f32 = 0.01;

/// Clamps bolt position to within the playfield walls and reflects the
/// offending velocity component.
///
/// Runs after all CCD collision systems. Only triggers when a bolt has
/// already escaped past a wall -- a belt-and-suspenders fix for the rare
/// case where CCD misses due to overlapping expanded AABBs at corners.
///
/// The bottom edge is intentionally open -- bolts that fall below the
/// playfield are handled by [`bolt_lost`].
pub(crate) fn clamp_bolt_to_playfield(
    playfield: Res<PlayfieldConfig>,
    mut bolt_query: Query<BoltCollisionData, ActiveFilter>,
) {
    for mut bolt in &mut bolt_query {
        let r = bolt.collision.radius.0 * bolt.collision.entity_scale.map_or(1.0, |s| s.0);
        let pos = bolt.spatial.position.0;

        let x_min = playfield.left() + r + BOUNDARY_INSET;
        let x_max = playfield.right() - r - BOUNDARY_INSET;
        let y_max = playfield.top() - r - BOUNDARY_INSET;

        let mut new_pos = pos;
        let mut new_vel = bolt.spatial.velocity.0;
        let mut clamped = false;

        if pos.x < x_min {
            new_pos.x = x_min;
            if new_vel.x < 0.0 {
                new_vel.x = -new_vel.x;
            }
            clamped = true;
        } else if pos.x > x_max {
            new_pos.x = x_max;
            if new_vel.x > 0.0 {
                new_vel.x = -new_vel.x;
            }
            clamped = true;
        }

        if pos.y > y_max {
            new_pos.y = y_max;
            if new_vel.y > 0.0 {
                new_vel.y = -new_vel.y;
            }
            clamped = true;
        }
        // No bottom clamp -- intentionally open for bolt-lost

        if clamped {
            bolt.spatial.position.0 = new_pos;
            bolt.spatial.velocity.0 = new_vel;

            // Apply the canonical velocity formula after clamping
            apply_velocity_formula(&mut bolt.spatial, bolt.collision.active_speed_boosts);
        }
    }
}
