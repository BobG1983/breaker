//! Distance constraint solver for tethered entity pairs.
//!
//! When two entities connected by a [`DistanceConstraint`] exceed the maximum
//! allowed distance, this system pulls them back symmetrically and redistributes
//! velocity along the constraint axis (momentum conservation).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::constraint::DistanceConstraint;

/// Enforces distance constraints between tethered entity pairs.
///
/// When the distance between two entities exceeds the constraint's `max_distance`,
/// both are pulled back symmetrically along the constraint axis, and their
/// velocity components along the axis are redistributed while perpendicular
/// velocity is preserved. Entities that are already converging keep their
/// velocities unchanged.
pub(crate) fn enforce_distance_constraints(
    constraint_query: Query<&DistanceConstraint>,
    mut entity_query: Query<(&mut Position2D, &mut Velocity2D)>,
) {
    for constraint in &constraint_query {
        let Ok([mut a, mut b]) =
            entity_query.get_many_mut([constraint.entity_a, constraint.entity_b])
        else {
            continue;
        };

        let delta = b.0.0 - a.0.0;
        let distance = delta.length();

        // Skip if same position (can't normalize zero vector) or within slack
        if distance < f32::EPSILON || distance <= constraint.max_distance {
            continue;
        }

        // Taut — apply position correction
        let axis = delta / distance;
        let half_correction = (distance - constraint.max_distance) / 2.0;
        a.0.0 += axis * half_correction;
        b.0.0 -= axis * half_correction;

        // Velocity redistribution — only when NOT both actively converging.
        // "Both converging" = A moving toward B (positive along axis) AND
        // B moving toward A (negative along axis). In that case both entities
        // will naturally close the gap and no velocity adjustment is needed.
        let dot_a = a.1.0.dot(axis);
        let dot_b = b.1.0.dot(axis);
        let both_converging = dot_a > 0.0 && dot_b < 0.0;

        if !both_converging {
            let avg = dot_a.midpoint(dot_b);
            a.1.0 += (avg - dot_a) * axis;
            b.1.0 += (avg - dot_b) * axis;
        }
    }
}
