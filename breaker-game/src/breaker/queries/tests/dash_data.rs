use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_spatial2d::components::MaxSpeed;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerDeceleration, BreakerTilt, DashDuration, DashSpeedMultiplier,
        DashState, DashStateTimer, DashTilt, DashTiltEase, DecelEasing, SettleDuration,
        SettleTiltEase,
    },
    prelude::*,
};

// ── Part D: BreakerDashData (mutable) ───────────────────────────

// Behavior 6: BreakerDashData mutable state, tilt, timer access
#[test]
fn breaker_dash_data_state_tilt_timer_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt {
                    angle:       10.0,
                    ease_start:  0.0,
                    ease_target: 10.0,
                },
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease:     EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle:    -5.0,
                    duration: 0.1,
                    ease:     EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
            ),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerDashData, With<Breaker>>| {
            for mut data in &mut query {
                *data.state = DashState::Braking;
                data.tilt.angle = -5.0;
                data.timer.remaining = 0.0;
            }
        },
    );
    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Braking);
    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!((tilt.angle - (-5.0)).abs() < f32::EPSILON);
    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
}

// Behavior 6: read-only config fields
#[test]
fn breaker_dash_data_readonly_config_fields() {
    let mut app = test_app();
    app.world_mut().spawn((
        (
            Breaker,
            DashState::Dashing,
            Velocity2D(Vec2::new(500.0, 0.0)),
            BreakerTilt {
                angle:       10.0,
                ease_start:  0.0,
                ease_target: 10.0,
            },
            DashStateTimer { remaining: 0.05 },
            MaxSpeed(600.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
        ),
        (
            DashSpeedMultiplier(2.0),
            DashDuration(0.15),
            DashTilt(15.0),
            DashTiltEase(EaseFunction::CubicIn),
            BrakeTilt {
                angle:    -5.0,
                duration: 0.1,
                ease:     EaseFunction::CubicIn,
            },
            BrakeDecel(3000.0),
            SettleDuration(0.1),
            SettleTiltEase(EaseFunction::CubicOut),
        ),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!((data.max_speed.0 - 600.0).abs() < f32::EPSILON);
                assert!((data.dash_speed.0 - 2.0).abs() < f32::EPSILON);
                assert!((data.dash_duration.0 - 0.15).abs() < f32::EPSILON);
                assert!((data.dash_tilt.0 - 15.0).abs() < f32::EPSILON);
                assert!((data.brake_tilt.angle - (-5.0)).abs() < f32::EPSILON);
                assert!((data.brake_decel.0 - 3000.0).abs() < f32::EPSILON);
                assert!((data.settle_duration.0 - 0.1).abs() < f32::EPSILON);
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 6 edge case: FlashStepActive present
#[test]
fn breaker_dash_data_flash_step_optional_present() {
    let mut app = test_app();
    app.world_mut().spawn((
        (
            Breaker,
            DashState::Dashing,
            Velocity2D(Vec2::new(500.0, 0.0)),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.05 },
            MaxSpeed(600.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
        ),
        (
            DashSpeedMultiplier(2.0),
            DashDuration(0.15),
            DashTilt(15.0),
            DashTiltEase(EaseFunction::CubicIn),
            BrakeTilt {
                angle:    -5.0,
                duration: 0.1,
                ease:     EaseFunction::CubicIn,
            },
            BrakeDecel(3000.0),
            SettleDuration(0.1),
            SettleTiltEase(EaseFunction::CubicOut),
            FlashStepActive,
        ),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!(data.flash_step.is_some(), "FlashStepActive should be Some");
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 7: BreakerDashData mutable position for flash step teleport
#[test]
fn breaker_dash_data_position_mutation_for_flash_step() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease:     EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle:    -5.0,
                    duration: 0.1,
                    ease:     EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
                FlashStepActive,
                Position2D(Vec2::new(0.0, -200.0)),
            ),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerDashData, With<Breaker>>| {
            for mut data in &mut query {
                if let Some(ref mut pos) = data.position {
                    pos.0.x = 200.0;
                }
            }
        },
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(200.0, -200.0));
}

// Behavior 7 edge case: position absent (None), no mutation possible
#[test]
fn breaker_dash_data_position_none_when_absent() {
    let mut app = test_app();
    app.world_mut().spawn((
        (
            Breaker,
            DashState::Dashing,
            Velocity2D(Vec2::new(500.0, 0.0)),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.05 },
            MaxSpeed(600.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
        ),
        (
            DashSpeedMultiplier(2.0),
            DashDuration(0.15),
            DashTilt(15.0),
            DashTiltEase(EaseFunction::CubicIn),
            BrakeTilt {
                angle:    -5.0,
                duration: 0.1,
                ease:     EaseFunction::CubicIn,
            },
            BrakeDecel(3000.0),
            SettleDuration(0.1),
            SettleTiltEase(EaseFunction::CubicOut),
            // No Position2D spawned
        ),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerDashDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                // Breaker's #[require(Spatial2D)] auto-inserts Position2D,
                // so it's always Some on a Breaker entity even when not explicitly spawned.
                assert!(
                    data.position.is_some(),
                    "Position2D auto-inserted via Breaker #[require]"
                );
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 8: velocity mutation through BreakerDashData
#[test]
fn breaker_dash_data_velocity_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            (
                Breaker,
                DashState::Dashing,
                Velocity2D(Vec2::new(500.0, 0.0)),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.05 },
                MaxSpeed(600.0),
                BreakerDeceleration(1500.0),
                DecelEasing {
                    ease:     EaseFunction::QuadraticIn,
                    strength: 1.0,
                },
            ),
            (
                DashSpeedMultiplier(2.0),
                DashDuration(0.15),
                DashTilt(15.0),
                DashTiltEase(EaseFunction::CubicIn),
                BrakeTilt {
                    angle:    -5.0,
                    duration: 0.1,
                    ease:     EaseFunction::CubicIn,
                },
                BrakeDecel(3000.0),
                SettleDuration(0.1),
                SettleTiltEase(EaseFunction::CubicOut),
            ),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerDashData, With<Breaker>>| {
            for mut data in &mut query {
                data.velocity.0 = Vec2::ZERO;
            }
        },
    );
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::ZERO);
}
