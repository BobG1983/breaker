//! Tests for `reset_breaker`: position reset, `PreviousPosition` match,
//! and full state restoration.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use super::super::system::*;
use crate::{
    breaker::{
        components::{Breaker, BreakerBaseY, BreakerTilt, BumpState, DashState, DashStateTimer},
        resources::BreakerConfig,
    },
    shared::{CleanupOnRunEnd, PlayfieldConfig},
};

#[test]
fn reset_breaker_writes_position2d() {
    // Given: Breaker at Position2D(Vec2::new(100.0, -200.0)), BreakerBaseY(-250.0)
    // When: reset_breaker runs
    // Then: Position2D(Vec2::new(0.0, -250.0))
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .init_resource::<PlayfieldConfig>();

    let config = BreakerConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -200.0)),
        PreviousPosition(Vec2::new(100.0, -200.0)),
        Velocity2D(Vec2::new(300.0, 0.0)),
        DashState::Dashing,
        BreakerTilt {
            angle: 0.5,
            ease_start: 0.5,
            ease_target: 0.0,
        },
        DashStateTimer { remaining: 0.1 },
        BreakerBaseY(config.y_position),
        BumpState {
            active: true,
            timer: 0.1,
            post_hit_timer: 0.05,
            cooldown: 0.2,
            last_hit_bolt: None,
        },
        CleanupOnRunEnd,
    ));

    app.add_systems(Update, reset_breaker);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("breaker should have Position2D");
    assert!(
        position.0.x.abs() < f32::EPSILON,
        "Position2D.x should be 0.0 after reset, got {}",
        position.0.x,
    );
    assert!(
        (position.0.y - config.y_position).abs() < f32::EPSILON,
        "Position2D.y should be {}, got {}",
        config.y_position,
        position.0.y,
    );
}

#[test]
fn reset_breaker_previous_position_matches_position() {
    // Given: Breaker at Position2D(Vec2::new(100.0, -200.0)) with stale PreviousPosition
    // When: reset_breaker runs
    // Then: PreviousPosition matches Position2D (no interpolation teleport)
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .init_resource::<PlayfieldConfig>();

    let config = BreakerConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, -200.0)),
        PreviousPosition(Vec2::new(50.0, -180.0)),
        Velocity2D(Vec2::new(300.0, 0.0)),
        DashState::Dashing,
        BreakerTilt {
            angle: 0.5,
            ease_start: 0.5,
            ease_target: 0.0,
        },
        DashStateTimer { remaining: 0.1 },
        BreakerBaseY(config.y_position),
        BumpState {
            active: true,
            timer: 0.1,
            post_hit_timer: 0.05,
            cooldown: 0.2,
            last_hit_bolt: None,
        },
        CleanupOnRunEnd,
    ));

    app.add_systems(Update, reset_breaker);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    let position = app
        .world()
        .get::<Position2D>(entity)
        .expect("breaker should have Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(entity)
        .expect("breaker should have PreviousPosition");
    assert_eq!(
        position.0, prev.0,
        "PreviousPosition should match Position2D after reset to prevent teleport"
    );
}

#[test]
fn reset_breaker_restores_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .init_resource::<PlayfieldConfig>();

    let config = BreakerConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(100.0, config.y_position + 50.0)),
        Velocity2D(Vec2::new(300.0, 0.0)),
        DashState::Dashing,
        BreakerTilt {
            angle: 0.5,
            ease_start: 0.5,
            ease_target: 0.0,
        },
        DashStateTimer { remaining: 0.1 },
        BreakerBaseY(config.y_position),
        BumpState {
            active: true,
            timer: 0.1,
            post_hit_timer: 0.05,
            cooldown: 0.2,
            last_hit_bolt: None,
        },
        CleanupOnRunEnd,
    ));

    app.add_systems(Update, reset_breaker);
    app.update();

    let (state, velocity, tilt, timer, bump) = app
        .world_mut()
        .query::<(
            &DashState,
            &Velocity2D,
            &BreakerTilt,
            &DashStateTimer,
            &BumpState,
        )>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    assert_eq!(*state, DashState::Idle);
    assert!(velocity.0.x.abs() < f32::EPSILON);
    assert!(tilt.angle.abs() < f32::EPSILON);
    assert!(tilt.ease_start.abs() < f32::EPSILON);
    assert!(timer.remaining.abs() < f32::EPSILON);
    assert!(!bump.active, "bump should be inactive after reset");
    assert!(
        bump.timer.abs() < f32::EPSILON,
        "bump timer should be cleared"
    );
    assert!(
        bump.post_hit_timer.abs() < f32::EPSILON,
        "post_hit_timer should be cleared"
    );
    assert!(
        bump.cooldown.abs() < f32::EPSILON,
        "cooldown should be cleared"
    );
}
