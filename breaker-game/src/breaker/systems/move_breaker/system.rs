//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, DashState},
        queries::MovementQuery,
    },
    effect::effects::{size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts},
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

/// Reads input actions and moves the breaker horizontally.
///
/// Accelerates toward max speed when movement is active, decelerates when released.
/// Movement is allowed in [`DashState::Idle`] and [`DashState::Settling`].
/// Clamps position to playfield bounds.
pub(crate) fn move_breaker(
    actions: Res<InputActions>,
    playfield: Res<PlayfieldConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<MovementQuery, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (
        mut position,
        mut velocity,
        state,
        max_speed,
        accel,
        decel,
        easing,
        half_width,
        speed_mult,
        size_mult,
    ) in &mut query
    {
        let effective_max = max_speed.0 * speed_mult.map_or(1.0, ActiveSpeedBoosts::multiplier);

        // Only allow direct input movement in Idle and Settling states
        let can_move = matches!(state, DashState::Idle | DashState::Settling);

        if can_move {
            let mut input_dir: f32 = 0.0;
            if actions.active(GameAction::MoveLeft) {
                input_dir -= 1.0;
            }
            if actions.active(GameAction::MoveRight) {
                input_dir += 1.0;
            }

            if input_dir.abs() > f32::EPSILON {
                // Accelerate toward input direction
                velocity.0.x = (input_dir * accel.0).mul_add(dt, velocity.0.x);
                velocity.0.x = velocity.0.x.clamp(-effective_max, effective_max);
            } else {
                // Decelerate toward zero with eased speed curve
                let effective_decel = super::super::dash::eased_decel(
                    decel.0,
                    velocity.0.x.abs(),
                    effective_max,
                    easing.ease,
                    easing.strength,
                );
                apply_deceleration(&mut velocity.0.x, effective_decel, dt);
            }
        }

        // Apply velocity to position
        position.0.x = velocity.0.x.mul_add(dt, position.0.x);

        // Clamp to playfield bounds (accounting for breaker effective half-width)
        let effective_half_w =
            half_width.half_width() * size_mult.map_or(1.0, ActiveSizeBoosts::multiplier);
        let min_x = playfield.left() + effective_half_w;
        let max_x = playfield.right() - effective_half_w;
        position.0.x = position.0.x.clamp(min_x, max_x);

        // Stop velocity if hitting a wall
        if position.0.x <= min_x || position.0.x >= max_x {
            velocity.0.x = 0.0;
        }
    }
}

/// Applies deceleration toward zero, clamping at zero to avoid oscillation.
pub(super) fn apply_deceleration(velocity: &mut f32, decel: f32, dt: f32) {
    if *velocity > f32::EPSILON {
        *velocity = decel.mul_add(-dt, *velocity).max(0.0);
    } else if *velocity < -f32::EPSILON {
        *velocity = decel.mul_add(dt, *velocity).min(0.0);
    }
}
