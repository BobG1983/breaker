use bevy::{math::curve::easing::EaseFunction, prelude::*};
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{
        BaseWidth, BreakerAcceleration, BreakerDeceleration, DashState, DecelEasing,
    },
    effect_v3::{
        effects::{SizeBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
    },
    prelude::*,
};

fn speed_stack(values: &[f32]) -> EffectStack<SpeedBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

fn size_stack(values: &[f32]) -> EffectStack<SizeBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

// ── Part C: BreakerMovementData (mutable) ───────────────────────

// Behavior 4: BreakerMovementData position mutation
#[test]
fn breaker_movement_data_position_mutation_takes_effect() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
            DashState::Idle,
            MaxSpeed(600.0),
            BreakerAcceleration(2000.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerMovementData, With<Breaker>>| {
            for mut data in &mut query {
                data.position.0.x += 10.0;
            }
        },
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(110.0, -200.0));
}

// Behavior 4: read-only config fields accessible
#[test]
fn breaker_movement_data_readonly_fields_accessible() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -200.0)),
        Velocity2D(Vec2::new(300.0, 0.0)),
        DashState::Idle,
        MaxSpeed(600.0),
        BreakerAcceleration(2000.0),
        BreakerDeceleration(1500.0),
        DecelEasing {
            ease:     EaseFunction::QuadraticIn,
            strength: 1.0,
        },
        BaseWidth(120.0),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerMovementDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert_eq!(*data.state, DashState::Idle);
                assert!((data.max_speed.0 - 600.0).abs() < f32::EPSILON);
                assert!((data.acceleration.0 - 2000.0).abs() < f32::EPSILON);
                assert!((data.deceleration.0 - 1500.0).abs() < f32::EPSILON);
                assert!((data.decel_easing.strength - 1.0).abs() < f32::EPSILON);
                assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 4 edge case: optional speed/size boosts
#[test]
fn breaker_movement_data_optional_boosts_present() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -200.0)),
        Velocity2D(Vec2::new(300.0, 0.0)),
        DashState::Idle,
        MaxSpeed(600.0),
        BreakerAcceleration(2000.0),
        BreakerDeceleration(1500.0),
        DecelEasing {
            ease:     EaseFunction::QuadraticIn,
            strength: 1.0,
        },
        BaseWidth(120.0),
        speed_stack(&[1.5]),
        size_stack(&[2.0]),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<BreakerMovementDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!(
                    data.speed_boosts.is_some(),
                    "ActiveSpeedBoosts should be Some"
                );
                assert!(
                    data.size_boosts.is_some(),
                    "ActiveSizeBoosts should be Some"
                );
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 5: velocity mutation takes effect
#[test]
fn breaker_movement_data_velocity_mutation_takes_effect() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
            DashState::Idle,
            MaxSpeed(600.0),
            BreakerAcceleration(2000.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerMovementData, With<Breaker>>| {
            for mut data in &mut query {
                data.velocity.0 = Vec2::new(500.0, 0.0);
            }
        },
    );
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::new(500.0, 0.0));
}

// Behavior 5 edge case: both position and velocity mutation in same invocation
#[test]
fn breaker_movement_data_both_position_and_velocity_mutable() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
            DashState::Idle,
            MaxSpeed(600.0),
            BreakerAcceleration(2000.0),
            BreakerDeceleration(1500.0),
            DecelEasing {
                ease:     EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<BreakerMovementData, With<Breaker>>| {
            for mut data in &mut query {
                data.position.0.x += 50.0;
                data.velocity.0 = Vec2::new(500.0, 0.0);
            }
        },
    );
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(150.0, -200.0));
    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert_eq!(vel.0, Vec2::new(500.0, 0.0));
}
