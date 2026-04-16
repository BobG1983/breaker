//! System that applies magnetic field forces to bolts.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use crate::{
    cells::behaviors::{
        magnetic::components::{MagneticCell, MagneticField},
        phantom::components::PhantomPhase,
    },
    prelude::*,
    shared::physics::inverse_square::inverse_square_attraction,
};

/// Applies inverse-square attraction from magnetic cells to nearby bolts.
///
/// For each bolt within a magnetic cell's radius, computes an attraction force
/// using `inverse_square_attraction`, caps the total acceleration at
/// `2 * base_speed`, and applies the velocity delta as `force * dt`.
///
/// Skips dead cells (`Without<Dead>` filter) and phantom cells in
/// `PhantomPhase::Ghost` (`Option<&PhantomPhase>` check). Only active in
/// `NodeState::Playing` (gated by the plugin's `run_if` condition).
type MagnetQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position2D,
        &'static MagneticField,
        &'static Aabb2D,
        Option<&'static PhantomPhase>,
    ),
    (With<MagneticCell>, Without<Dead>),
>;

pub(crate) fn apply_magnetic_fields(
    time: Res<Time<Fixed>>,
    magnet_query: MagnetQuery,
    mut bolt_query: Query<(&mut Velocity2D, &Position2D, &BaseSpeed), With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (mut vel, bolt_pos, base_speed) in &mut bolt_query {
        let mut total_force = Vec2::ZERO;
        for (cell_pos, field, aabb, phantom) in &magnet_query {
            if phantom.is_some_and(|p| *p == PhantomPhase::Ghost) {
                continue;
            }
            let dist = bolt_pos.0.distance(cell_pos.0);
            if dist > field.radius {
                continue;
            }
            let min_dist = aabb.half_extents.x;
            total_force +=
                inverse_square_attraction(cell_pos.0, bolt_pos.0, field.strength, min_dist);
        }
        let max_accel = 2.0 * base_speed.0;
        total_force = total_force.clamp_length_max(max_accel);
        vel.0 += total_force * dt;
    }
}
