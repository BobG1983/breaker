//! Dash, brake, and settle state machine systems.

use bevy::prelude::*;

use crate::breaker::components::{
    Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity,
};
use crate::breaker::resources::BreakerConfig;

/// Handles dash input and the Dashing → Braking → Settling → Idle state machine.
///
/// - Dash input (Space): triggers dash from Idle or Settling
/// - Dashing: high-speed burst with tilt, transitions to Braking when timer expires
/// - Braking: rapid deceleration with opposite tilt, transitions to Settling when speed ~0
/// - Settling: tilt returns to neutral, transitions to Idle when timer expires
pub fn update_breaker_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<BreakerConfig>,
    time: Res<Time<Fixed>>,
    mut query: Query<
        (
            &mut BreakerState,
            &mut BreakerVelocity,
            &mut BreakerTilt,
            &mut BreakerStateTimer,
        ),
        With<Breaker>,
    >,
) {
    let dt = time.delta_secs();

    for (mut state, mut velocity, mut tilt, mut timer) in &mut query {
        match *state {
            BreakerState::Idle | BreakerState::Settling => {
                handle_idle_or_settling(
                    &keyboard,
                    &config,
                    dt,
                    &mut state,
                    &mut velocity,
                    &mut tilt,
                    &mut timer,
                );
            }
            BreakerState::Dashing => {
                handle_dashing(&config, dt, &mut state, &velocity, &mut tilt, &mut timer);
            }
            BreakerState::Braking => {
                handle_braking(&config, dt, &mut state, &mut velocity, &tilt);
            }
        }
    }
}

/// In Idle or Settling: check for dash input.
fn handle_idle_or_settling(
    keyboard: &ButtonInput<KeyCode>,
    config: &BreakerConfig,
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
) {
    if *state == BreakerState::Settling {
        // Tick settle timer, return tilt toward zero
        timer.remaining -= dt;
        let settle_progress = if config.settle_duration > f32::EPSILON {
            1.0 - (timer.remaining / config.settle_duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        tilt.angle *= 1.0 - settle_progress;

        if timer.remaining <= 0.0 {
            *state = BreakerState::Idle;
            tilt.angle = 0.0;
        }
    }

    // Dash input: LShift or RShift
    if keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::ShiftRight) {
        // Need a movement direction to dash — use current velocity or input
        let dash_dir = if velocity.x.abs() > f32::EPSILON {
            velocity.x.signum()
        } else if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            -1.0
        } else if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            1.0
        } else {
            return; // No direction — can't dash
        };

        *state = BreakerState::Dashing;
        velocity.x = dash_dir * config.max_speed * config.dash_speed_multiplier;
        tilt.angle = dash_dir * config.dash_tilt_angle;
        timer.remaining = config.dash_duration;
    }
}

/// Dashing: count down timer, then transition to Braking.
fn handle_dashing(
    config: &BreakerConfig,
    dt: f32,
    state: &mut BreakerState,
    velocity: &BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
) {
    timer.remaining -= dt;

    // Maintain tilt in dash direction
    let dash_dir = velocity.x.signum();
    tilt.angle = dash_dir * config.dash_tilt_angle;

    if timer.remaining <= 0.0 {
        *state = BreakerState::Braking;
        // Reverse tilt for braking
        tilt.angle = -dash_dir * config.brake_tilt_angle;
    }
}

/// Braking: rapidly decelerate, then transition to Settling.
fn handle_braking(
    config: &BreakerConfig,
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    _tilt: &BreakerTilt,
) {
    let brake_decel = config.deceleration * config.brake_decel_multiplier;

    if velocity.x > f32::EPSILON {
        velocity.x = brake_decel.mul_add(-dt, velocity.x).max(0.0);
    } else if velocity.x < -f32::EPSILON {
        velocity.x = brake_decel.mul_add(dt, velocity.x).min(0.0);
    }

    // Speed near zero → transition to Settling
    if velocity.x.abs() <= f32::EPSILON {
        velocity.x = 0.0;
        *state = BreakerState::Settling;
        // Tilt carries over from braking, will be eased back to zero
    }

    // Tilt was set when entering Braking; stays until Settling eases it back.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::components::BreakerState;

    fn spawn_test_breaker(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                Breaker,
                BreakerState::Idle,
                BreakerVelocity { x: 0.0 },
                BreakerTilt { angle: 0.0 },
                BreakerStateTimer { remaining: 0.0 },
            ))
            .id()
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<ButtonInput<KeyCode>>();
        // Use Update instead of FixedUpdate for unit tests to avoid
        // timing issues with fixed timestep accumulation.
        app.add_systems(Update, update_breaker_state);
        app
    }

    #[test]
    fn idle_stays_idle_without_input() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);
        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Idle);
    }

    #[test]
    fn dash_transitions_from_idle() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        // Set velocity so there's a direction to dash
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 100.0;

        // Simulate shift press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Dashing);
    }

    #[test]
    fn dash_sets_tilt() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        // Moving right, then dash
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 100.0;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.update();

        let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
        assert!(tilt.angle > 0.0, "dashing right should tilt right");
    }

    #[test]
    fn cannot_dash_without_direction() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        // No velocity, no directional input
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(
            *state,
            BreakerState::Idle,
            "should not dash without direction"
        );
    }

    #[test]
    fn dashing_transitions_to_braking() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        // Enter dashing state manually
        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Dashing;
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 500.0;
        app.world_mut()
            .get_mut::<BreakerStateTimer>(entity)
            .unwrap()
            .remaining = 0.0;

        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Braking);
    }

    #[test]
    fn braking_transitions_to_settling() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        // Enter braking with near-zero velocity
        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Braking;
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 0.0;

        app.update();

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Settling);
    }
}
