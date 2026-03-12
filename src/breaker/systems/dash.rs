//! Dash, brake, and settle state machine systems.

use bevy::{
    math::curve::{Curve, easing::EaseFunction},
    prelude::*,
};

use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerMaxSpeed, BreakerState,
            BreakerStateTimer, BreakerTilt, BreakerVelocity, DashDuration, DashSpeedMultiplier,
            DashTilt, DecelEasing, SettleDuration, SettleTiltEase,
        },
        queries::BreakerDashQuery,
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
    time: Res<Time<Fixed>>,
    mut query: Query<BreakerDashQuery, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (
        mut state,
        mut velocity,
        mut tilt,
        mut timer,
        max_speed,
        decel,
        easing,
        dash_speed,
        dash_duration,
        dash_tilt,
        brake_tilt,
        brake_decel,
        settle_duration,
        settle_tilt_ease,
    ) in &mut query
    {
        match *state {
            BreakerState::Idle | BreakerState::Settling => {
                handle_idle_or_settling(
                    &actions,
                    dt,
                    &mut state,
                    &mut velocity,
                    &mut tilt,
                    &mut timer,
                    max_speed,
                    dash_speed,
                    dash_duration,
                    dash_tilt,
                    settle_duration,
                    settle_tilt_ease,
                );
            }
            BreakerState::Dashing => {
                handle_dashing(
                    dt, &mut state, &velocity, &mut tilt, &mut timer, dash_tilt, brake_tilt,
                );
            }
            BreakerState::Braking => {
                handle_braking(
                    dt,
                    &mut state,
                    &mut velocity,
                    &mut tilt,
                    max_speed,
                    decel,
                    easing,
                    dash_speed,
                    brake_decel,
                );
            }
        }
    }
}

/// In Idle or Settling: check for dash actions from the input domain.
#[allow(clippy::too_many_arguments)]
fn handle_idle_or_settling(
    actions: &InputActions,
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    max_speed: &BreakerMaxSpeed,
    dash_speed: &DashSpeedMultiplier,
    dash_duration: &DashDuration,
    dash_tilt: &DashTilt,
    settle_duration: &SettleDuration,
    settle_tilt_ease: &SettleTiltEase,
) {
    if *state == BreakerState::Settling {
        // Tick settle timer, return tilt toward zero with easing
        timer.remaining -= dt;
        let settle_progress = if settle_duration.0 > f32::EPSILON {
            1.0 - (timer.remaining / settle_duration.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = settle_tilt_ease.0.sample_clamped(settle_progress);
        tilt.angle = tilt.settle_start_angle * (1.0 - eased);

        if timer.remaining <= 0.0 {
            *state = BreakerState::Idle;
            tilt.angle = 0.0;
        }
    }

    // Dash left
    if actions.active(GameAction::DashLeft) {
        start_dash(
            -1.0,
            state,
            velocity,
            tilt,
            timer,
            max_speed,
            dash_speed,
            dash_duration,
            dash_tilt,
        );
        return;
    }

    // Dash right
    if actions.active(GameAction::DashRight) {
        start_dash(
            1.0,
            state,
            velocity,
            tilt,
            timer,
            max_speed,
            dash_speed,
            dash_duration,
            dash_tilt,
        );
    }
}

/// Enters the Dashing state in the given direction.
#[allow(clippy::too_many_arguments)]
fn start_dash(
    direction: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    max_speed: &BreakerMaxSpeed,
    dash_speed: &DashSpeedMultiplier,
    dash_duration: &DashDuration,
    dash_tilt: &DashTilt,
) {
    *state = BreakerState::Dashing;
    velocity.x = direction * max_speed.0 * dash_speed.0;
    tilt.angle = direction * dash_tilt.0;
    timer.remaining = dash_duration.0;
}

/// Dashing: count down timer, then transition to Braking.
fn handle_dashing(
    dt: f32,
    state: &mut BreakerState,
    velocity: &BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    dash_tilt: &DashTilt,
    brake_tilt: &BrakeTilt,
) {
    timer.remaining -= dt;

    // Maintain tilt in dash direction
    let dash_dir = velocity.x.signum();
    tilt.angle = dash_dir * dash_tilt.0;

    if timer.remaining <= 0.0 {
        *state = BreakerState::Braking;
        // Reverse tilt for braking
        tilt.angle = -dash_dir * brake_tilt.0;
    }
}

/// Braking: rapidly decelerate with eased speed curve, then transition to Settling.
#[allow(clippy::too_many_arguments)]
fn handle_braking(
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    max_speed: &BreakerMaxSpeed,
    decel: &BreakerDeceleration,
    easing: &DecelEasing,
    dash_speed: &DashSpeedMultiplier,
    brake_decel: &BrakeDecel,
) {
    let base_decel = decel.0 * brake_decel.0;
    let reference_speed = max_speed.0 * dash_speed.0;
    let effective_decel = eased_decel(
        base_decel,
        velocity.x.abs(),
        reference_speed,
        easing.ease,
        easing.strength,
    );

    if velocity.x > f32::EPSILON {
        velocity.x = effective_decel.mul_add(-dt, velocity.x).max(0.0);
    } else if velocity.x < -f32::EPSILON {
        velocity.x = effective_decel.mul_add(dt, velocity.x).min(0.0);
    }

    // Speed near zero → transition to Settling
    if velocity.x.abs() <= f32::EPSILON {
        velocity.x = 0.0;
        tilt.settle_start_angle = tilt.angle;
        *state = BreakerState::Settling;
    }
}

/// Computes effective deceleration scaled by an easing curve over speed ratio.
///
/// `effective_decel = base_decel * (1.0 + strength * ease(speed / reference_speed))`
pub fn eased_decel(
    base_decel: f32,
    speed: f32,
    reference_speed: f32,
    ease: EaseFunction,
    strength: f32,
) -> f32 {
    let speed_ratio = if reference_speed > f32::EPSILON {
        (speed / reference_speed).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let factor = ease.sample_clamped(speed_ratio);
    base_decel * strength.mul_add(factor, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::{components::BreakerState, resources::BreakerConfig};

    fn breaker_param_bundle(
        config: &BreakerConfig,
    ) -> (
        BreakerMaxSpeed,
        BreakerDeceleration,
        DecelEasing,
        DashSpeedMultiplier,
        DashDuration,
        DashTilt,
        BrakeTilt,
        BrakeDecel,
        SettleDuration,
        SettleTiltEase,
    ) {
        (
            BreakerMaxSpeed(config.max_speed),
            BreakerDeceleration(config.deceleration),
            DecelEasing {
                ease: config.decel_ease,
                strength: config.decel_ease_strength,
            },
            DashSpeedMultiplier(config.dash_speed_multiplier),
            DashDuration(config.dash_duration),
            DashTilt(config.dash_tilt_angle),
            BrakeTilt(config.brake_tilt_angle),
            BrakeDecel(config.brake_decel_multiplier),
            SettleDuration(config.settle_duration),
            SettleTiltEase(config.settle_tilt_ease),
        )
    }

    fn spawn_test_breaker(app: &mut App) -> Entity {
        let config = BreakerConfig::default();
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
                breaker_param_bundle(&config),
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
    fn settling_tilt_eased_not_linear() {
        let mut app = test_app();
        let entity = spawn_test_breaker(&mut app);
        let config = BreakerConfig::default();

        let start_angle = 0.44;
        *app.world_mut().get_mut::<BreakerState>(entity).unwrap() = BreakerState::Settling;
        {
            let mut tilt = app.world_mut().get_mut::<BreakerTilt>(entity).unwrap();
            tilt.angle = start_angle;
            tilt.settle_start_angle = start_angle;
        }
        app.world_mut()
            .get_mut::<BreakerStateTimer>(entity)
            .unwrap()
            .remaining = config.settle_duration;

        // Advance to ~50% of settle duration
        let dt = std::time::Duration::from_secs_f64(f64::from(config.settle_duration) * 0.5);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut().resource_mut::<Time<Fixed>>().advance_by(dt);
        app.update();

        let angle = app.world().get::<BreakerTilt>(entity).unwrap().angle;
        // With CubicOut at 50% progress, result is 0.875 (much further than linear 0.5)
        // So angle should be well below 50% of start_angle (0.22)
        let linear_50pct = start_angle * 0.5;
        assert!(
            angle < linear_50pct,
            "CubicOut settle at 50% progress should be well below linear 50% ({linear_50pct}), got {angle}"
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

    // ── eased_decel unit tests ────────────────────────────────────────

    #[test]
    fn eased_decel_stronger_at_high_speed() {
        use bevy::math::curve::easing::EaseFunction;

        let base = 1000.0;
        let reference = 500.0;
        let ease = EaseFunction::QuadraticIn;
        let strength = 1.0;

        let decel_low = eased_decel(base, 50.0, reference, ease, strength);
        let decel_high = eased_decel(base, 450.0, reference, ease, strength);

        assert!(
            decel_high > decel_low,
            "decel at high speed ({decel_high}) should exceed decel at low speed ({decel_low})"
        );
    }

    #[test]
    fn eased_decel_reaches_zero() {
        use bevy::math::curve::easing::EaseFunction;

        // At zero speed, QuadraticIn(0) = 0, so effective = base * (1 + 1 * 0) = base
        let decel = eased_decel(1000.0, 0.0, 500.0, EaseFunction::QuadraticIn, 1.0);
        assert!(
            (decel - 1000.0).abs() < f32::EPSILON,
            "at zero speed, decel should equal base, got {decel}"
        );
    }

    #[test]
    fn zero_strength_matches_constant_decel() {
        use bevy::math::curve::easing::EaseFunction;

        let base = 1000.0;
        let decel = eased_decel(base, 400.0, 500.0, EaseFunction::QuadraticIn, 0.0);
        assert!(
            (decel - base).abs() < f32::EPSILON,
            "zero strength should give constant base decel, got {decel}"
        );
    }
}
