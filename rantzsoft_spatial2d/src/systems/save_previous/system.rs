//! Snapshots current position/rotation/scale to previous for interpolation.

use bevy::prelude::*;

use crate::components::*;

/// Copies current global (or local fallback) position, rotation, and scale
/// into their previous-frame snapshots for entities that have
/// `InterpolateTransform2D`. Also snapshots `Velocity2D` to
/// `PreviousVelocity`.
pub(crate) fn save_previous(
    mut query_pos: Query<
        (
            &Position2D,
            Option<&GlobalPosition2D>,
            &mut PreviousPosition,
        ),
        With<InterpolateTransform2D>,
    >,
    mut query_rot: Query<
        (
            &Rotation2D,
            Option<&GlobalRotation2D>,
            &mut PreviousRotation,
        ),
        With<InterpolateTransform2D>,
    >,
    mut query_scale: Query<
        (&Scale2D, Option<&GlobalScale2D>, &mut PreviousScale),
        With<InterpolateTransform2D>,
    >,
    mut query_vel: Query<(&Velocity2D, &mut PreviousVelocity), With<InterpolateTransform2D>>,
) {
    for (pos, global_pos, mut prev) in &mut query_pos {
        prev.0 = global_pos.map_or(pos.0, |g| g.0);
    }
    for (rot, global_rot, mut prev) in &mut query_rot {
        prev.0 = global_rot.map_or(rot.0, |g| g.0);
    }
    for (scale, global_scale, mut prev) in &mut query_scale {
        if let Some(g) = global_scale {
            prev.x = g.x;
            prev.y = g.y;
        } else {
            prev.x = scale.x;
            prev.y = scale.y;
        }
    }
    for (vel, mut prev) in &mut query_vel {
        prev.0 = vel.0;
    }
}
