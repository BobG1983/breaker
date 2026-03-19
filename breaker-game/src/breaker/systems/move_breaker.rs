//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, BreakerState},
        queries::MovementQuery,
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
    playfield: Res<PlayfieldConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<MovementQuery, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (
        mut transform,
        mut velocity,
        state,
        max_speed,
        accel,
        decel,
        easing,
        half_width,
        speed_boost,
        width_boost,
    ) in &mut query
    {
        let effective_max = max_speed.0 + speed_boost.map_or(0.0, |b| b.0);

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
                velocity.x = (input_dir * accel.0).mul_add(dt, velocity.x);
                velocity.x = velocity.x.clamp(-effective_max, effective_max);
            } else {
                // Decelerate toward zero with eased speed curve
                let effective_decel = super::dash::eased_decel(
                    decel.0,
                    velocity.x.abs(),
                    effective_max,
                    easing.ease,
                    easing.strength,
                );
                apply_deceleration(&mut velocity.x, effective_decel, dt);
            }
        }

        // Apply velocity to position
        transform.translation.x = velocity.x.mul_add(dt, transform.translation.x);

        // Clamp to playfield bounds (accounting for breaker effective half-width)
        let effective_half_w = half_width.half_width() + width_boost.map_or(0.0, |b| b.0 / 2.0);
        let min_x = playfield.left() + effective_half_w;
        let max_x = playfield.right() - effective_half_w;
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
    use crate::{
        breaker::{
            components::{
                BreakerAcceleration, BreakerDeceleration, BreakerMaxSpeed, BreakerState,
                BreakerVelocity, BreakerWidth, DecelEasing,
            },
            resources::BreakerConfig,
        },
        chips::components::{BreakerSpeedBoost, WidthBoost},
    };

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
        app.add_plugins(MinimalPlugins)
            .init_resource::<PlayfieldConfig>()
            .init_resource::<InputActions>()
            .add_systems(FixedUpdate, move_breaker);
        app
    }

    /// Accumulates one fixed timestep of overstep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_breaker(app: &mut App, state: BreakerState) -> Entity {
        let config = BreakerConfig::default();
        app.world_mut()
            .spawn((
                Breaker,
                state,
                BreakerVelocity { x: 0.0 },
                BreakerMaxSpeed(config.max_speed),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
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
        let half_width = app
            .world()
            .get::<BreakerWidth>(entity)
            .unwrap()
            .half_width();

        // Push breaker far past right boundary
        app.world_mut()
            .get_mut::<Transform>(entity)
            .unwrap()
            .translation
            .x = 9999.0;
        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        let max_x = playfield.right() - half_width;
        assert!(
            tf.translation.x <= max_x + f32::EPSILON,
            "breaker should be clamped to playfield, got x={} max={}",
            tf.translation.x,
            max_x
        );
    }

    #[test]
    fn speed_boost_raises_effective_max_speed() {
        // Given: BreakerMaxSpeed(500.0) + BreakerSpeedBoost(100.0), velocity.x = 590.0
        //        MoveRight input active (so the acceleration+clamp path runs)
        // When: move_breaker system runs
        // Then: velocity.x NOT clamped to 500 — effective max is 600, so velocity stays > 500
        //
        // Current code clamps to max_speed.0 = 500 (ignores BreakerSpeedBoost) → test FAILS
        let mut app = integration_app();
        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 590.0 },
                BreakerMaxSpeed(500.0),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
                BreakerSpeedBoost(100.0),
                Transform::from_xyz(0.0, config.y_position, 0.0),
            ))
            .id();

        // MoveRight input — ensures the acceleration path (and clamp) runs.
        // Current code: velocity clamped to max_speed.0 = 500 (ignores boost).
        // Expected: clamped to max_speed.0 + boost = 600, so velocity stays > 500.
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(
            vel.x > 500.0 + f32::EPSILON,
            "velocity {:.3} should NOT be clamped to 500 when BreakerSpeedBoost(100.0) makes effective max 600",
            vel.x
        );
    }

    #[test]
    fn no_speed_boost_base_max_speed_clamps_velocity() {
        // Regression guard: without BreakerSpeedBoost, velocity above max_speed IS clamped.
        // Given: BreakerMaxSpeed(500.0), no BreakerSpeedBoost, velocity.x = 600.0
        // When: move_breaker runs
        // Then: velocity.x is clamped to 500.0
        let mut app = integration_app();
        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 600.0 },
                BreakerMaxSpeed(500.0),
                BreakerAcceleration(0.0),
                BreakerDeceleration(0.0),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
                // No BreakerSpeedBoost
                Transform::from_xyz(0.0, config.y_position, 0.0),
            ))
            .id();

        // Provide MoveRight input so the acceleration path (with clamp) runs
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(
            vel.x <= 500.0 + f32::EPSILON,
            "velocity {:.3} should be clamped to base max_speed 500.0 with no boost",
            vel.x
        );
    }

    #[test]
    fn width_boost_increases_effective_half_width_for_clamping() {
        // Given: BreakerWidth(120.0), WidthBoost(40.0), PlayfieldConfig default (right=400)
        //        effective half_w = (120+40)/2 = 80
        //        Breaker placed far right (9999.0) — position will be clamped during tick
        // When: move_breaker runs
        // Then: clamped to max_x = 400 - 80 = 320
        //        (without boost: max_x = 400 - 60 = 340; current code clamps to 340, not 320)
        //
        // Current code uses half_width.half_width() (ignores WidthBoost) → clamps to 340 → FAILS
        let mut app = integration_app();
        let config = BreakerConfig::default();

        // PlayfieldConfig default: width=800, so right()=400
        let playfield = app.world().resource::<PlayfieldConfig>().clone();
        assert!(
            (playfield.right() - 400.0).abs() < 1.0,
            "test assumes default playfield right=400, got {}",
            playfield.right()
        );

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 0.0 },
                BreakerMaxSpeed(config.max_speed),
                BreakerAcceleration(0.0),
                BreakerDeceleration(0.0),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(120.0),
                WidthBoost(40.0),
                // Place far right — clamping must apply during tick
                Transform::from_xyz(9999.0, config.y_position, 0.0),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        // With WidthBoost: effective half_width = (120 + 40) / 2 = 80
        //   max_x = 400 - 80 = 320
        // Without WidthBoost (current behavior): max_x = 400 - 60 = 340
        //   current code clamps to 340, so x = 340, which is > 320 → assertion fails
        let expected_max_x = 320.0_f32;
        assert!(
            tf.translation.x <= expected_max_x + f32::EPSILON,
            "with WidthBoost effective half_w=80, x {:.3} should be clamped to {:.3}, not to base {:.3}",
            tf.translation.x,
            expected_max_x,
            400.0 - 60.0
        );
    }
}
