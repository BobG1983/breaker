//! Dash, brake, and settle state machine systems.

use bevy::prelude::*;

use crate::{
    breaker::{
        components::{Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity},
        resources::BreakerConfig,
    },
    input::resources::{GameAction, InputActions},
};

/// Handles dash input and the Dashing → Braking → Settling → Idle state machine.
///
/// - Dash input (`DashLeft`/`DashRight` from input domain): triggers dash from Idle or Settling
/// - Dashing: high-speed burst with tilt, transitions to Braking when timer expires
/// - Braking: rapid deceleration with opposite tilt, transitions to Settling when speed ~0
/// - Settling: tilt returns to neutral, transitions to Idle when timer expires
pub fn update_breaker_state(
    actions: Res<InputActions>,
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
                    &actions,
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
                handle_braking(&config, dt, &mut state, &mut velocity, &mut tilt);
            }
        }
    }
}

/// In Idle or Settling: check for dash actions from the input domain.
fn handle_idle_or_settling(
    actions: &InputActions,
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
        tilt.angle = tilt.settle_start_angle * (1.0 - settle_progress);

        if timer.remaining <= 0.0 {
            *state = BreakerState::Idle;
            tilt.angle = 0.0;
        }
    }

    // Dash left
    if actions.active(GameAction::DashLeft) {
        start_dash(-1.0, config, state, velocity, tilt, timer);
        return;
    }

    // Dash right
    if actions.active(GameAction::DashRight) {
        start_dash(1.0, config, state, velocity, tilt, timer);
    }
}

/// Enters the Dashing state in the given direction.
fn start_dash(
    direction: f32,
    config: &BreakerConfig,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
) {
    *state = BreakerState::Dashing;
    velocity.x = direction * config.max_speed * config.dash_speed_multiplier;
    tilt.angle = direction * config.dash_tilt_angle;
    timer.remaining = config.dash_duration;
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
    tilt: &mut BreakerTilt,
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
        tilt.settle_start_angle = tilt.angle;
        *state = BreakerState::Settling;
    }
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
                BreakerTilt {
                    angle: 0.0,
                    settle_start_angle: 0.0,
                },
                BreakerStateTimer { remaining: 0.0 },
            ))
            .id()
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<InputActions>();
        app.add_systems(Update, update_breaker_state);
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

    #[test]
    fn idle_stays_idle_without_input() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);
        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Idle);
    }

    #[test]
    fn dash_left_triggers_dashing() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::DashLeft);
        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Dashing);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(vel.x < 0.0, "dash left should have negative velocity");
    }

    #[test]
    fn dash_right_triggers_dashing() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::DashRight);
        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Dashing);

        let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
        assert!(vel.x > 0.0, "dash right should have positive velocity");
    }

    #[test]
    fn dash_right_sets_tilt() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::DashRight);
        tick(&mut app);

        let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
        assert!(tilt.angle > 0.0, "dashing right should tilt right");
    }

    #[test]
    fn dashing_transitions_to_braking() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Dashing;
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 500.0;
        app.world_mut()
            .get_mut::<BreakerStateTimer>(entity)
            .unwrap()
            .remaining = 0.0;

        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Braking);
    }

    #[test]
    fn settling_transitions_to_idle_and_resets_tilt() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Settling;
        {
            let mut tilt = app.world_mut().get_mut::<BreakerTilt>(entity).unwrap();
            tilt.angle = 0.3;
            tilt.settle_start_angle = 0.3;
        }
        app.world_mut()
            .get_mut::<BreakerStateTimer>(entity)
            .unwrap()
            .remaining = 0.0;

        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(
            *state,
            BreakerState::Idle,
            "settling should transition to idle when timer expires"
        );

        let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
        assert!(
            tilt.angle.abs() < f32::EPSILON,
            "tilt should be reset to zero after settling, got {}",
            tilt.angle
        );
    }

    #[test]
    fn settling_tilt_is_frame_rate_independent() {
        use std::time::Duration;

        let start_angle = 0.44;
        let config = BreakerConfig::default();
        let settle_dur = config.settle_duration;

        let dt_60 = Duration::from_secs_f64(1.0 / 60.0);
        let steps_60: u32 = 3;
        let dt_240 = Duration::from_secs_f64(1.0 / 240.0);
        let steps_240: u32 = 12;

        let mut app_60 = test_app();
        let e60 = spawn_test_breaker(&mut app_60);
        *app_60.world_mut().get_mut::<BreakerState>(e60).unwrap() = BreakerState::Settling;
        {
            let mut tilt = app_60.world_mut().get_mut::<BreakerTilt>(e60).unwrap();
            tilt.angle = start_angle;
            tilt.settle_start_angle = start_angle;
        }
        app_60
            .world_mut()
            .get_mut::<BreakerStateTimer>(e60)
            .unwrap()
            .remaining = settle_dur;
        app_60
            .world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt_60);
        for _ in 0..steps_60 {
            app_60
                .world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(dt_60);
            app_60.update();
        }
        let angle_60 = app_60.world().get::<BreakerTilt>(e60).unwrap().angle;

        let mut app_240 = test_app();
        let e240 = spawn_test_breaker(&mut app_240);
        *app_240.world_mut().get_mut::<BreakerState>(e240).unwrap() = BreakerState::Settling;
        {
            let mut tilt = app_240.world_mut().get_mut::<BreakerTilt>(e240).unwrap();
            tilt.angle = start_angle;
            tilt.settle_start_angle = start_angle;
        }
        app_240
            .world_mut()
            .get_mut::<BreakerStateTimer>(e240)
            .unwrap()
            .remaining = settle_dur;
        app_240
            .world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt_240);
        for _ in 0..steps_240 {
            app_240
                .world_mut()
                .resource_mut::<Time<Fixed>>()
                .advance_by(dt_240);
            app_240.update();
        }
        let angle_240 = app_240.world().get::<BreakerTilt>(e240).unwrap().angle;

        assert!(
            (angle_60 - angle_240).abs() < 0.001,
            "tilt should be frame-rate independent: 60fps={angle_60}, 240fps={angle_240}"
        );
    }

    #[test]
    fn braking_transitions_to_settling() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);

        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Braking;
        app.world_mut()
            .get_mut::<BreakerVelocity>(entity)
            .unwrap()
            .x = 0.0;

        tick(&mut app);

        let state = app.world().get::<BreakerState>(entity).unwrap();
        assert_eq!(*state, BreakerState::Settling);
    }
}
