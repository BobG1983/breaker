//! System to prepare bolt velocity each fixed tick.
//!
//! Enforces speed clamping and minimum angle. Does NOT update position --
//! the CCD system in the physics domain handles position advancement.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::*, filters::ActiveFilter},
    breaker::components::{Breaker, MinAngleFromHorizontal},
    chips::components::BoltSpeedBoost,
};

/// Prepares the bolt velocity for the current timestep.
///
/// Enforces speed clamping (min/max) and minimum angle from horizontal
/// on `Velocity2D`.
/// Position advancement is handled by the CCD collision system.
pub(crate) fn prepare_bolt_velocity(
    mut query: Query<
        (
            &mut Velocity2D,
            &BoltMinSpeed,
            &BoltMaxSpeed,
            Option<&BoltSpeedBoost>,
        ),
        ActiveFilter,
    >,
    breaker_query: Query<&MinAngleFromHorizontal, (With<Breaker>, Without<Bolt>)>,
) {
    let Ok(min_angle) = breaker_query.single() else {
        return;
    };

    for (mut velocity, min_speed, max_speed, speed_boost) in &mut query {
        let boost = speed_boost.map_or(0.0, |b| b.0);
        let effective_min = min_speed.0 + boost;
        let effective_max = max_speed.0 + boost;

        let speed = velocity.speed();
        if speed > f32::EPSILON {
            velocity.0 = velocity.0.clamp_length(effective_min, effective_max);
            enforce_min_angle(&mut velocity.0, min_angle.0);
        }
    }
}
