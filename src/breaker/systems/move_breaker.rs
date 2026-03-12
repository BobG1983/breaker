//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, BreakerState, BreakerVelocity},
        resources::BreakerConfig,
    },
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

/// Reads input actions and moves the breaker horizontally.
///
/// Accelerates toward max speed when movement is active, decelerates when released.
/// Movement is allowed in [`BreakerState::Idle`] and [`BreakerState::Settling`].
/// Clamps position to playfield bounds.
pub fn move_breaker(
    actions: Res<InputActions>,
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
            if actions.active(GameAction::MoveLeft) {
                input_dir -= 1.0;
            }
            if actions.active(GameAction::MoveRight) {
                input_dir += 1.0;
            }

            if input_dir.abs() > f32::EPSILON {
                // Accelerate toward input direction
                velocity.x = (input_dir * config.acceleration).mul_add(dt, velocity.x);
                velocity.x = velocity.x.clamp(-config.max_speed, config.max_speed);
            } else {
                // Decelerate toward zero with eased speed curve
                let effective_decel = super::dash::eased_decel(
                    config.deceleration,
                    velocity.x.abs(),
                    config.max_speed,
                    config.decel_ease,
                    config.decel_ease_strength,
                );
                apply_deceleration(&mut velocity.x, effective_decel, dt);
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

    fn integration_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<PlayfieldConfig>();
        app.init_resource::<InputActions>();
        app.add_systems(Update, move_breaker);
        app
    }

    /// Advances `Time<Fixed>` by one default timestep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();
    }

    fn spawn_breaker(app: &mut App, state: BreakerState) -> Entity {
        let config = app.world().resource::<BreakerConfig>().clone();
        app.world_mut()
            .spawn((
                Breaker,
                state,
                BreakerVelocity { x: 0.0 },
                Transform::from_xyz(0.0, config.y_position, 0.0),
            ))
            .id()
    }

    #[test]
    fn right_input_moves_breaker_right() {
        let mut app = integration_app();
        let entity = spawn_breaker(&mut app, BreakerState::Idle);

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert!(
            tf.translation.x > 0.0,
            "breaker should move right, got x={}",
            tf.translation.x
        );
    }

    #[test]
    fn dashing_blocks_input_acceleration() {
        let mut app = integration_app();
        let entity = spawn_breaker(&mut app, BreakerState::Dashing);

        // Set velocity to zero, then push MoveRight — Dashing should not accelerate
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(
            vel.x.abs() < f32::EPSILON,
            "dashing state should not accelerate from keyboard, got vx={}",
            vel.x
        );
    }

    #[test]
    fn position_clamped_to_playfield_bounds() {
        let mut app = integration_app();
        let entity = spawn_breaker(&mut app, BreakerState::Idle);
        let playfield = app.world().resource::<PlayfieldConfig>().clone();
        let config = app.world().resource::<BreakerConfig>().clone();

        // Push breaker far past right boundary
        app.world_mut()
            .get_mut::<Transform>(entity)
            .unwrap()
            .translation
            .x = 9999.0;
        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let max_x = playfield.right() - config.half_width;
        assert!(
            tf.translation.x <= max_x + f32::EPSILON,
            "breaker should be clamped to playfield, got x={} max={}",
            tf.translation.x,
            max_x
        );
    }
}
