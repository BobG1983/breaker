//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::{
    breaker::{components::DashState, queries::BreakerMovementData},
    effect_v3::{effects::*, stacking::EffectStack},
    input::resources::GameAction,
    prelude::*,
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
    mut query: Query<BreakerMovementData, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for mut data in &mut query {
        let effective_max = data.max_speed.0
            * data
                .speed_boosts
                .map_or(1.0, EffectStack::<SpeedBoostConfig>::aggregate);

        // Only allow direct input movement in Idle and Settling states
        let can_move = matches!(*data.state, DashState::Idle | DashState::Settling);

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
                data.velocity.0.x =
                    (input_dir * data.acceleration.0).mul_add(dt, data.velocity.0.x);
                data.velocity.0.x = data.velocity.0.x.clamp(-effective_max, effective_max);
            } else {
                // Decelerate toward zero with eased speed curve
                let effective_decel = crate::breaker::systems::dash::eased_decel(
                    data.deceleration.0,
                    data.velocity.0.x.abs(),
                    effective_max,
                    data.decel_easing.ease,
                    data.decel_easing.strength,
                );
                apply_deceleration(&mut data.velocity.0.x, effective_decel, dt);
            }
        }

        // Apply velocity to position
        data.position.0.x = data.velocity.0.x.mul_add(dt, data.position.0.x);

        // Clamp to playfield bounds (accounting for breaker effective half-width)
        let effective_half_w = data.base_width.half_width()
            * data
                .size_boosts
                .map_or(1.0, EffectStack::<SizeBoostConfig>::aggregate);
        let min_x = playfield.left() + effective_half_w;
        let max_x = playfield.right() - effective_half_w;
        data.position.0.x = data.position.0.x.clamp(min_x, max_x);

        // Stop velocity if hitting a wall
        if data.position.0.x <= min_x || data.position.0.x >= max_x {
            data.velocity.0.x = 0.0;
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
