//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::breaker::components::{Breaker, BreakerState, BreakerVelocity};
use crate::breaker::resources::BreakerConfig;
use crate::shared::PlayfieldConfig;

/// Reads keyboard input and moves the breaker horizontally.
///
/// Accelerates toward max speed when input is held, decelerates when released.
/// Movement is allowed in [`BreakerState::Idle`] and [`BreakerState::Settling`].
/// Clamps position to playfield bounds.
pub fn move_breaker(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<BreakerConfig>,
    playfield: Res<PlayfieldConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut BreakerVelocity, &BreakerState), With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity, state) in &mut query {
        // Only allow direct input movement in Idle and Settling states
        let can_move = matches!(state, BreakerState::Idle | BreakerState::Settling);

        if can_move {
            let mut input_dir: f32 = 0.0;
            if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
                input_dir -= 1.0;
            }
            if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
                input_dir += 1.0;
            }

            if input_dir.abs() > f32::EPSILON {
                // Accelerate toward input direction
                velocity.x = (input_dir * config.acceleration).mul_add(dt, velocity.x);
                velocity.x = velocity.x.clamp(-config.max_speed, config.max_speed);
            } else {
                // Decelerate toward zero
                apply_deceleration(&mut velocity.x, config.deceleration, dt);
            }
        }

        // Apply velocity to position
        transform.translation.x = velocity.x.mul_add(dt, transform.translation.x);

        // Clamp to playfield bounds (accounting for breaker half-width)
        let min_x = playfield.left() + config.half_width;
        let max_x = playfield.right() - config.half_width;
        transform.translation.x = transform.translation.x.clamp(min_x, max_x);

        // Stop velocity if hitting a wall
        if transform.translation.x <= min_x || transform.translation.x >= max_x {
            velocity.x = 0.0;
        }
    }
}

/// Applies deceleration toward zero, clamping at zero to avoid oscillation.
fn apply_deceleration(velocity: &mut f32, decel: f32, dt: f32) {
    if *velocity > f32::EPSILON {
        *velocity = decel.mul_add(-dt, *velocity).max(0.0);
    } else if *velocity < -f32::EPSILON {
        *velocity = decel.mul_add(dt, *velocity).min(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deceleration_toward_zero_positive() {
        let mut vel = 100.0;
        apply_deceleration(&mut vel, 500.0, 0.1);
        assert!((vel - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn deceleration_toward_zero_negative() {
        let mut vel = -100.0;
        apply_deceleration(&mut vel, 500.0, 0.1);
        assert!((vel - (-50.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn deceleration_clamps_at_zero() {
        let mut vel = 10.0;
        apply_deceleration(&mut vel, 500.0, 1.0);
        assert!((vel - 0.0).abs() < f32::EPSILON);
    }
}
