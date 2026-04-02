use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{super::system::*, helpers::*};
use crate::{
    breaker::{
        components::{BreakerTilt, DashState, DashStateTimer},
        definition::BreakerDefinition,
    },
    input::resources::{GameAction, InputActions},
};

#[test]
fn idle_stays_idle_without_input() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);
    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle);
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

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Dashing);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(vel.0.x < 0.0, "dash left should have negative velocity");
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

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Dashing);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(vel.0.x > 0.0, "dash right should have positive velocity");
}

#[test]
fn dash_right_sets_tilt() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    // First tick enters Dashing (tilt starts at 0), second tick eases tilt
    tick(&mut app);
    tick(&mut app);

    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!(tilt.angle > 0.0, "dashing right should tilt right");
}

#[test]
fn dashing_transitions_to_braking() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);

    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Dashing;
    app.world_mut().get_mut::<Velocity2D>(entity).unwrap().0.x = 500.0;
    app.world_mut()
        .get_mut::<DashStateTimer>(entity)
        .unwrap()
        .remaining = 0.0;

    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Braking);
}

#[test]
fn settling_transitions_to_idle_and_resets_tilt() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);

    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Settling;
    {
        let mut tilt = app.world_mut().get_mut::<BreakerTilt>(entity).unwrap();
        tilt.angle = 0.3;
        tilt.ease_start = 0.3;
        tilt.ease_target = 0.0;
    }
    app.world_mut()
        .get_mut::<DashStateTimer>(entity)
        .unwrap()
        .remaining = 0.0;

    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Idle,
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
    let config = BreakerDefinition::default();
    let settle_dur = config.settle_duration;

    let dt_60 = Duration::from_secs_f64(1.0 / 60.0);
    let steps_60: u32 = 3;
    let dt_240 = Duration::from_secs_f64(1.0 / 240.0);
    let steps_240: u32 = 12;

    let mut app_60 = test_app();
    let e60 = spawn_test_breaker(&mut app_60);
    *app_60.world_mut().get_mut::<DashState>(e60).unwrap() = DashState::Settling;
    {
        let mut tilt = app_60.world_mut().get_mut::<BreakerTilt>(e60).unwrap();
        tilt.angle = start_angle;
        tilt.ease_start = start_angle;
        tilt.ease_target = 0.0;
    }
    app_60
        .world_mut()
        .get_mut::<DashStateTimer>(e60)
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
            .accumulate_overstep(dt_60);
        app_60.update();
    }
    let angle_60 = app_60.world().get::<BreakerTilt>(e60).unwrap().angle;

    let mut app_240 = test_app();
    let e240 = spawn_test_breaker(&mut app_240);
    *app_240.world_mut().get_mut::<DashState>(e240).unwrap() = DashState::Settling;
    {
        let mut tilt = app_240.world_mut().get_mut::<BreakerTilt>(e240).unwrap();
        tilt.angle = start_angle;
        tilt.ease_start = start_angle;
        tilt.ease_target = 0.0;
    }
    app_240
        .world_mut()
        .get_mut::<DashStateTimer>(e240)
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
            .accumulate_overstep(dt_240);
        app_240.update();
    }
    let angle_240 = app_240.world().get::<BreakerTilt>(e240).unwrap().angle;

    assert!(
        (angle_60 - angle_240).abs() < 0.02,
        "tilt should be frame-rate independent: 60fps={angle_60}, 240fps={angle_240}"
    );
}

#[test]
fn settling_tilt_eased_not_linear() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);
    let config = BreakerDefinition::default();

    let start_angle = 0.44;
    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Settling;
    {
        let mut tilt = app.world_mut().get_mut::<BreakerTilt>(entity).unwrap();
        tilt.angle = start_angle;
        tilt.ease_start = start_angle;
        tilt.ease_target = 0.0;
    }
    app.world_mut()
        .get_mut::<DashStateTimer>(entity)
        .unwrap()
        .remaining = config.settle_duration;

    // Advance to ~50% of settle duration
    let dt = std::time::Duration::from_secs_f64(f64::from(config.settle_duration) * 0.5);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
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

    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Braking;
    app.world_mut().get_mut::<Velocity2D>(entity).unwrap().0.x = 0.0;

    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Settling);
}

#[test]
fn brake_tilt_eases_not_snaps() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);
    let config = BreakerDefinition::default();

    let dash_tilt_angle = config.dash_tilt_angle.to_radians();
    let brake_tilt_angle = config.brake_tilt_angle.to_radians();

    // Set up mid-dash with tilt at full dash angle, timer about to expire
    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Dashing;
    app.world_mut().get_mut::<Velocity2D>(entity).unwrap().0.x =
        config.max_speed * config.dash_speed_multiplier;
    app.world_mut()
        .get_mut::<BreakerTilt>(entity)
        .unwrap()
        .angle = dash_tilt_angle;
    app.world_mut()
        .get_mut::<DashStateTimer>(entity)
        .unwrap()
        .remaining = 0.0;

    // Tick once: transitions Dashing -> Braking (tilt unchanged)
    tick(&mut app);
    assert_eq!(
        *app.world().get::<DashState>(entity).unwrap(),
        DashState::Braking
    );

    // Tick again: first frame of brake tilt easing
    tick(&mut app);
    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();

    // Tilt should NOT have snapped to the full brake angle
    assert!(
        (tilt.angle - (-brake_tilt_angle)).abs() > 0.01,
        "brake tilt should ease gradually, not snap to full angle ({:.3}), got {:.3}",
        -brake_tilt_angle,
        tilt.angle
    );
    // Tilt should have started moving away from dash angle
    assert!(
        tilt.angle < dash_tilt_angle,
        "tilt should have moved from dash angle ({dash_tilt_angle:.3}), got {:.3}",
        tilt.angle
    );
}

#[test]
fn settle_timer_initialized_on_braking_end() {
    let mut app = test_app();
    let entity = spawn_test_breaker(&mut app);

    // Start in Braking with zero velocity (will immediately transition)
    *app.world_mut().get_mut::<DashState>(entity).unwrap() = DashState::Braking;
    app.world_mut().get_mut::<Velocity2D>(entity).unwrap().0.x = 0.0;

    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Settling);

    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    // Timer should have been initialized to settle_duration minus one dt
    assert!(
        timer.remaining > 0.0,
        "settle timer should be initialized with positive remaining time, got {}",
        timer.remaining
    );

    // Settling should NOT immediately transition to Idle
    tick(&mut app);
    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Settling,
        "settling should persist for multiple frames, not finish instantly"
    );

    // After enough time, should reach Idle
    // Tick well past the settle duration to ensure state reaches Idle
    for _ in 0..100 {
        tick(&mut app);
    }
    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle);
}

// -- eased_decel unit tests ----------------------------------------

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
