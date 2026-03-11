//! System to move the bolt by its velocity each fixed tick.

use bevy::prelude::*;

use crate::bolt::components::{Bolt, BoltVelocity};
use crate::bolt::resources::BoltConfig;

/// Moves the bolt by its velocity each fixed timestep.
///
/// Enforces speed clamping and minimum angle from horizontal.
pub fn move_bolt(
    config: Res<BoltConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut BoltVelocity), With<Bolt>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity) in &mut query {
        // Enforce speed bounds
        let speed = velocity.speed();
        if speed > f32::EPSILON {
            let clamped_speed = speed.clamp(config.min_speed, config.max_speed);
            if (clamped_speed - speed).abs() > f32::EPSILON {
                velocity.value = velocity.direction() * clamped_speed;
            }

            // Enforce minimum angle from horizontal
            velocity.enforce_min_angle(config.min_angle_from_horizontal);
        }

        // Apply velocity to position
        transform.translation.x = velocity.value.x.mul_add(dt, transform.translation.x);
        transform.translation.y = velocity.value.y.mul_add(dt, transform.translation.y);
    }
}
