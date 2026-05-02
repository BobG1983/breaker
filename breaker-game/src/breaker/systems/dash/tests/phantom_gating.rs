use bevy::prelude::*;

use super::helpers::{breaker_param_bundle, test_app};
use crate::{
    breaker::{
        builder::core::types::BreakerPhantomParams,
        components::{BreakerTilt, DashState, DashStateTimer, PhantomBreaker},
        test_utils::default_breaker_definition,
    },
    input::resources::GameAction,
    prelude::*,
};

// ── Wave 3 Behavior 3: DashLeft drives real but NOT phantom ──

#[test]
fn update_breaker_state_dash_input_drives_real_only() {
    let def = default_breaker_definition();
    let mut app = test_app();

    let real = {
        let world = app.world_mut();
        let e = world
            .spawn((
                Breaker,
                DashState::Idle,
                Velocity2D(Vec2::ZERO),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.0 },
                breaker_param_bundle(&def),
            ))
            .id();
        world.flush();
        e
    };
    let phantom = {
        let world = app.world_mut();
        let e = Breaker::builder()
            .definition(&def)
            .phantom(BreakerPhantomParams {
                lifespan:          2.5,
                phantom_color_rgb: [0.4, 0.8, 1.0],
                flicker_frequency: 4.0,
                flicker_min_alpha: 0.3,
            })
            .headless()
            .extra()
            .spawn(&mut world.commands());
        world.flush();
        e
    };

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_some(),
        "phantom entity must have PhantomBreaker marker"
    );

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let real_state = app.world().get::<DashState>(real).unwrap();
    assert_eq!(
        *real_state,
        DashState::Dashing,
        "real breaker should transition to DashState::Dashing on DashLeft input"
    );

    let real_vel = app.world().get::<Velocity2D>(real).unwrap();
    assert!(
        real_vel.0.x < 0.0,
        "real breaker should have negative velocity after DashLeft, got {}",
        real_vel.0.x
    );

    let phantom_state = app.world().get::<DashState>(phantom).unwrap();
    assert_eq!(
        *phantom_state,
        DashState::Idle,
        "phantom breaker must remain Idle after DashLeft input (update_breaker_state must not process phantom)"
    );

    let phantom_vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        phantom_vel.0.x.abs() < f32::EPSILON,
        "phantom velocity must remain zero after DashLeft, got {}",
        phantom_vel.0.x
    );

    let phantom_timer = app.world().get::<DashStateTimer>(phantom).unwrap();
    assert!(
        phantom_timer.remaining.abs() < f32::EPSILON,
        "phantom DashStateTimer.remaining must be 0.0 after tick (no dash-enter side effect), got {}",
        phantom_timer.remaining
    );
}

// ── Wave 3 Behavior 4: DashRight drives real but NOT phantom ──

#[test]
fn update_breaker_state_dash_input_drives_real_only_right() {
    let def = default_breaker_definition();
    let mut app = test_app();

    let real = {
        let world = app.world_mut();
        let e = world
            .spawn((
                Breaker,
                DashState::Idle,
                Velocity2D(Vec2::ZERO),
                BreakerTilt::default(),
                DashStateTimer { remaining: 0.0 },
                breaker_param_bundle(&def),
            ))
            .id();
        world.flush();
        e
    };
    let phantom = {
        let world = app.world_mut();
        let e = Breaker::builder()
            .definition(&def)
            .phantom(BreakerPhantomParams {
                lifespan:          2.5,
                phantom_color_rgb: [0.4, 0.8, 1.0],
                flicker_frequency: 4.0,
                flicker_min_alpha: 0.3,
            })
            .headless()
            .extra()
            .spawn(&mut world.commands());
        world.flush();
        e
    };

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_some(),
        "phantom entity must have PhantomBreaker marker"
    );

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let real_state = app.world().get::<DashState>(real).unwrap();
    assert_eq!(
        *real_state,
        DashState::Dashing,
        "real breaker should transition to DashState::Dashing on DashRight input"
    );

    let real_vel = app.world().get::<Velocity2D>(real).unwrap();
    assert!(
        real_vel.0.x > 0.0,
        "real breaker should have positive velocity after DashRight, got {}",
        real_vel.0.x
    );

    let phantom_state = app.world().get::<DashState>(phantom).unwrap();
    assert_eq!(
        *phantom_state,
        DashState::Idle,
        "phantom breaker must remain Idle after DashRight input (update_breaker_state must not process phantom)"
    );

    let phantom_vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        phantom_vel.0.x.abs() < f32::EPSILON,
        "phantom velocity must remain zero after DashRight, got {}",
        phantom_vel.0.x
    );

    let phantom_timer = app.world().get::<DashStateTimer>(phantom).unwrap();
    assert!(
        phantom_timer.remaining.abs() < f32::EPSILON,
        "phantom DashStateTimer.remaining must be 0.0 after tick (no dash-enter side effect), got {}",
        phantom_timer.remaining
    );
}
