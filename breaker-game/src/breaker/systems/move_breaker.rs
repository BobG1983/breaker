//! System to move the breaker based on input.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, BreakerState},
        queries::MovementQuery,
    },
    effect::effects::{size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts},
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

/// Reads input actions and moves the breaker horizontally.
///
/// Accelerates toward max speed when movement is active, decelerates when released.
/// Movement is allowed in [`BreakerState::Idle`] and [`BreakerState::Settling`].
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
        position.0.x = velocity.x.mul_add(dt, position.0.x);

        // Clamp to playfield bounds (accounting for breaker effective half-width)
        let effective_half_w =
            half_width.half_width() * size_mult.map_or(1.0, ActiveSizeBoosts::multiplier);
        let min_x = playfield.left() + effective_half_w;
        let max_x = playfield.right() - effective_half_w;
        position.0.x = position.0.x.clamp(min_x, max_x);

        // Stop velocity if hitting a wall
        if position.0.x <= min_x || position.0.x >= max_x {
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
    use rantzsoft_spatial2d::components::{MaxSpeed, Position2D};

    use super::*;
    use crate::{
        breaker::{
            components::{
                BreakerAcceleration, BreakerDeceleration, BreakerState, BreakerVelocity,
                BreakerWidth, DecelEasing,
            },
            resources::BreakerConfig,
        },
        effect::effects::{size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts},
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

    fn spawn_breaker_at(app: &mut App, state: BreakerState, position: Vec2) -> Entity {
        let config = BreakerConfig::default();
        app.world_mut()
            .spawn((
                Breaker,
                state,
                BreakerVelocity { x: 0.0 },
                MaxSpeed(config.max_speed),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
                Position2D(position),
            ))
            .id()
    }

    fn spawn_breaker(app: &mut App, state: BreakerState) -> Entity {
        let config = BreakerConfig::default();
        spawn_breaker_at(app, state, Vec2::new(0.0, config.y_position))
    }

    #[test]
    fn right_input_moves_breaker_right() {
        // Given: Breaker in Idle state, BreakerVelocity { x: 0.0 },
        //        Position2D(Vec2::new(0.0, -250.0)), dt=1/60
        // When: move_breaker runs with MoveRight input
        // Then: Position2D.0.x > 0.0
        let mut app = integration_app();
        let entity = spawn_breaker(&mut app, BreakerState::Idle);

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert!(
            pos.0.x > 0.0,
            "breaker should move right, got Position2D.x={}",
            pos.0.x
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
        // Given: Breaker Position2D(Vec2::new(9999.0, -250.0)), BreakerWidth(120.0),
        //        playfield right=400
        // When: move_breaker runs
        // Then: Position2D.0.x <= 340.0 (400 - 60)
        let mut app = integration_app();
        let config = BreakerConfig::default();
        let entity = spawn_breaker_at(
            &mut app,
            BreakerState::Idle,
            Vec2::new(9999.0, config.y_position),
        );
        let playfield = app.world().resource::<PlayfieldConfig>().clone();
        let half_width = app
            .world()
            .get::<BreakerWidth>(entity)
            .unwrap()
            .half_width();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        let max_x = playfield.right() - half_width;
        assert!(
            pos.0.x <= max_x + f32::EPSILON,
            "breaker should be clamped to playfield, got Position2D.x={} max={}",
            pos.0.x,
            max_x
        );
    }

    #[test]
    fn speed_multiplier_raises_effective_max_speed() {
        // Given: MaxSpeed(500.0) + ActiveSpeedBoosts(vec![1.2]), velocity.x = 590.0
        //        MoveRight input active (so the acceleration+clamp path runs)
        // When: move_breaker system runs
        // Then: velocity.x > 500 AND velocity.x <= 600 (effective max = 500 * 1.2 = 600)
        let mut app = integration_app();
        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 590.0 },
                MaxSpeed(500.0),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
                ActiveSpeedBoosts(vec![1.2]),
                Position2D(Vec2::new(0.0, config.y_position)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(
            vel.x > 500.0 + f32::EPSILON,
            "velocity {:.3} should NOT be clamped to base 500 when ActiveSpeedBoosts([1.2]) makes effective max 600",
            vel.x
        );
        assert!(
            vel.x <= 600.0 + f32::EPSILON,
            "velocity {:.3} should be clamped to effective max 600.0 (500 * 1.2)",
            vel.x
        );
    }

    #[test]
    fn no_speed_boost_base_max_speed_clamps_velocity() {
        // Regression guard: without BreakerSpeedBoost, velocity above max_speed IS clamped.
        let mut app = integration_app();
        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 600.0 },
                MaxSpeed(500.0),
                BreakerAcceleration(0.0),
                BreakerDeceleration(0.0),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(config.width),
                Position2D(Vec2::new(0.0, config.y_position)),
            ))
            .id();

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
    fn size_multiplier_increases_effective_half_width_for_clamping() {
        // Given: BreakerWidth(120.0), ActiveSizeBoosts(vec![4/3]), PlayfieldConfig default (right=400)
        //        effective half_w = 60.0 * (4/3) = 80
        //        Breaker placed far right (9999.0) — position will be clamped during tick
        // When: move_breaker runs
        // Then: Position2D.0.x clamped to max_x = 400 - 80 = 320
        let mut app = integration_app();
        let config = BreakerConfig::default();

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
                MaxSpeed(config.max_speed),
                BreakerAcceleration(0.0),
                BreakerDeceleration(0.0),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                BreakerWidth(120.0),
                ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
                Position2D(Vec2::new(9999.0, config.y_position)),
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        let expected_max_x = 320.0_f32;
        assert!(
            pos.0.x <= expected_max_x + f32::EPSILON,
            "with ActiveSizeBoosts([4/3]) effective half_w=80, Position2D.x {:.3} should be clamped to {:.3}, not to base {:.3}",
            pos.0.x,
            expected_max_x,
            400.0 - 60.0
        );
    }

    #[test]
    fn move_breaker_reads_active_speed_boosts_for_max_speed() {
        // Given: Breaker with ActiveSpeedBoosts(vec![2.0]), MaxSpeed(300.0),
        //        BreakerVelocity { x: 590.0 }, MoveRight active
        // When: move_breaker runs
        // Then: velocity.x clamped to 600.0 (300.0 * 2.0), not 300.0
        let mut app = integration_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 590.0 },
                MaxSpeed(300.0),
                BreakerAcceleration(5000.0),
                BreakerDeceleration(3000.0),
                DecelEasing {
                    ease: bevy::math::curve::easing::EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
                BreakerWidth(120.0),
                ActiveSpeedBoosts(vec![2.0]),
                Position2D(Vec2::new(0.0, -250.0)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::MoveRight);
        tick(&mut app);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(
            vel.x > 300.0 + f32::EPSILON,
            "velocity {:.3} should exceed base max 300.0 with ActiveSpeedBoosts([2.0]) → effective max 600.0",
            vel.x
        );
        assert!(
            vel.x <= 600.0 + f32::EPSILON,
            "velocity {:.3} should be clamped to effective max 600.0 (300 * 2.0)",
            vel.x
        );
    }

    #[test]
    fn move_breaker_reads_active_size_boosts_for_playfield_clamping() {
        // Given: Breaker with ActiveSizeBoosts(vec![2.0]), BreakerWidth(120.0) (half_width=60.0),
        //        Position2D far right (9999.0)
        // When: move_breaker clamps position
        // Then: position clamped using effective_half_w = 60.0 * 2.0 = 120.0
        //       -> max_x = 400.0 - 120.0 = 280.0
        let mut app = integration_app();
        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 0.0 },
                MaxSpeed(300.0),
                BreakerAcceleration(0.0),
                BreakerDeceleration(0.0),
                DecelEasing {
                    ease: bevy::math::curve::easing::EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
                BreakerWidth(120.0),
                ActiveSizeBoosts(vec![2.0]),
                Position2D(Vec2::new(9999.0, -250.0)),
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        let expected_max_x = 280.0_f32; // 400.0 - (60.0 * 2.0)
        assert!(
            pos.0.x <= expected_max_x + f32::EPSILON,
            "with ActiveSizeBoosts([2.0]) effective half_w=120.0, Position2D.x {:.3} should be clamped to {:.3}",
            pos.0.x,
            expected_max_x
        );
    }
}
