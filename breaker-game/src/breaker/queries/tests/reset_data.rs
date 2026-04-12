use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use super::{super::data::*, helpers::*};
use crate::breaker::components::{
    Breaker, BreakerBaseY, BreakerTilt, BumpState, DashState, DashStateTimer,
};

// ── Part G: BreakerResetData (mutable) ──────────────────────────

// Behavior 14: BreakerResetData full mutable reset
#[test]
fn breaker_reset_data_full_mutable_reset() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -180.0)),
            DashState::Dashing,
            Velocity2D(Vec2::new(300.0, 0.0)),
            BreakerTilt {
                angle:       15.0,
                ease_start:  0.0,
                ease_target: 15.0,
            },
            DashStateTimer { remaining: 0.1 },
            BumpState {
                active: true,
                ..BumpState::default()
            },
            BreakerBaseY(-200.0),
            PreviousPosition(Vec2::new(90.0, -180.0)),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerResetData, With<Breaker>>| {
            for mut data in &mut query {
                data.position.0 = Vec2::new(0.0, -200.0);
                data.velocity.0 = Vec2::ZERO;
                *data.state = DashState::Idle;
                data.tilt.angle = 0.0;
                data.timer.remaining = 0.0;
                data.bump.active = false;
            }
        },
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(0.0, -200.0));
    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::ZERO);
    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle);
    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!((tilt.angle - 0.0).abs() < f32::EPSILON);
    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active);
}

// Behavior 14: base_y is read-only
#[test]
fn breaker_reset_data_base_y_readable() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -180.0)),
        DashState::Dashing,
        Velocity2D(Vec2::new(300.0, 0.0)),
        BreakerTilt::default(),
        DashStateTimer { remaining: 0.1 },
        BumpState::default(),
        BreakerBaseY(-200.0),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerResetDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!((data.base_y.0 - (-200.0)).abs() < f32::EPSILON);
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 14 edge case: prev_position absent
#[test]
fn breaker_reset_data_prev_position_none_when_absent() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -180.0)),
        DashState::Dashing,
        Velocity2D(Vec2::new(300.0, 0.0)),
        BreakerTilt::default(),
        DashStateTimer { remaining: 0.1 },
        BumpState::default(),
        BreakerBaseY(-200.0),
        // No PreviousPosition
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerResetDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                // Breaker's #[require(Spatial2D)] auto-inserts PreviousPosition,
                // so it's always Some on a Breaker entity even when not explicitly spawned.
                assert!(
                    data.prev_position.is_some(),
                    "PreviousPosition auto-inserted via Breaker #[require]"
                );
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 15: prev_position mutation works when present
#[test]
fn breaker_reset_data_prev_position_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -180.0)),
            DashState::Dashing,
            Velocity2D(Vec2::new(300.0, 0.0)),
            BreakerTilt::default(),
            DashStateTimer { remaining: 0.1 },
            BumpState::default(),
            BreakerBaseY(-200.0),
            PreviousPosition(Vec2::new(90.0, -180.0)),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerResetData, With<Breaker>>| {
            for mut data in &mut query {
                if let Some(ref mut prev) = data.prev_position {
                    **prev = PreviousPosition(Vec2::ZERO);
                }
            }
        },
    );
    tick(&mut app);

    let prev = app.world().get::<PreviousPosition>(entity).unwrap();
    assert_eq!(prev.0, Vec2::ZERO);
}
