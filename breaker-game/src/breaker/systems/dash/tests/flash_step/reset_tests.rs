use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::{
        components::{BaseWidth, BreakerTilt, DashState, DashStateTimer},
        definition::BreakerDefinition,
    },
    input::resources::GameAction,
    prelude::*,
};

// -- Behavior 10: Teleport resets tilt and timer cleanly to Idle ----

#[test]
fn flash_step_teleport_resets_tilt_and_timer_to_idle() {
    // Given: Settling with partially eased tilt (angle=-0.25, ease_start=-0.35, ease_target=0.0),
    //        timer remaining=0.15, FlashStepActive, reversal dash input
    // When: update_breaker_state runs
    // Then: tilt.angle == 0.0, tilt.ease_start == 0.0, tilt.ease_target == 0.0,
    //       timer.remaining == 0.0, DashState == Idle
    let mut app = test_app();
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle:       -0.25,
                ease_start:  -0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.15 },
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
            FlashStepActive,
            breaker_param_bundle(&config),
        ))
        .id();

    // ease_start < 0 means last dash was rightward; DashLeft = reversal
    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle, "teleport should reset to Idle");

    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!(
        tilt.angle.abs() < f32::EPSILON,
        "teleport should reset tilt.angle to 0.0, got {}",
        tilt.angle
    );
    assert!(
        tilt.ease_start.abs() < f32::EPSILON,
        "teleport should reset tilt.ease_start to 0.0, got {}",
        tilt.ease_start
    );
    assert!(
        tilt.ease_target.abs() < f32::EPSILON,
        "teleport should reset tilt.ease_target to 0.0, got {}",
        tilt.ease_target
    );

    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    assert!(
        timer.remaining.abs() < f32::EPSILON,
        "teleport should reset timer.remaining to 0.0, got {}",
        timer.remaining
    );
}

#[test]
fn flash_step_teleport_resets_cleanly_with_nearly_expired_timer() {
    // Edge case: timer nearly expired (0.001) still resets cleanly
    let mut app = test_app();
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle:       -0.25,
                ease_start:  -0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.001 },
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
            FlashStepActive,
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(
        *state,
        DashState::Idle,
        "nearly expired timer should still produce clean Idle reset"
    );

    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!(
        tilt.angle.abs() < f32::EPSILON,
        "tilt.angle should be 0.0 even with nearly expired timer, got {}",
        tilt.angle
    );

    let timer = app.world().get::<DashStateTimer>(entity).unwrap();
    assert!(
        timer.remaining.abs() < f32::EPSILON,
        "timer.remaining should be 0.0, got {}",
        timer.remaining
    );
}
