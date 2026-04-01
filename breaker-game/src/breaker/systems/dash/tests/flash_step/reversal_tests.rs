use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D};

use super::helpers::*;
use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerState, BreakerStateTimer,
            BreakerTilt, BreakerVelocity, BreakerWidth, DashDuration, DashSpeedMultiplier,
            DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
        },
        resources::BreakerConfig,
    },
    effect::effects::flash_step::FlashStepActive,
    input::resources::{GameAction, InputActions},
};

// -- Behavior 1: Reversal dash right-to-left teleports breaker ------

#[test]
fn reversal_dash_left_during_settling_with_flash_step_teleports_to_endpoint() {
    // Given: Breaker in Settling at (0.0, -250.0), last dash rightward (ease_start=-0.35),
    //        FlashStepActive, MaxSpeed(500), DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashLeft active
    // Then: Position2D.x == -300.0, BreakerState == Idle, velocity.x == 0.0
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(0.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-300.0)).abs() < f32::EPSILON,
        "teleport should set position to -300.0 (0 + (-1)*500*4*0.15), got {}",
        pos.0.x
    );

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Idle,
        "teleport should transition directly to Idle"
    );

    let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
    assert!(
        vel.x.abs() < f32::EPSILON,
        "teleport should zero velocity, got {}",
        vel.x
    );
}

// -- Behavior 2: Reversal dash left-to-right also teleports ---------

#[test]
fn reversal_dash_right_during_settling_with_flash_step_teleports_to_endpoint() {
    // Given: Breaker in Settling at (0.0, -250.0), last dash leftward (ease_start=0.35),
    //        FlashStepActive, MaxSpeed(500), DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashRight active
    // Then: Position2D.x == 300.0, BreakerState == Idle, velocity.x == 0.0
    let mut app = test_app();
    let entity = spawn_settling_breaker_leftward_dash(&mut app, Vec2::new(0.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 300.0).abs() < f32::EPSILON,
        "teleport should set position to 300.0 (0 + 1*500*4*0.15), got {}",
        pos.0.x
    );

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Idle,
        "teleport should transition directly to Idle"
    );

    let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
    assert!(
        vel.x.abs() < f32::EPSILON,
        "teleport should zero velocity, got {}",
        vel.x
    );
}

#[test]
fn reversal_dash_right_with_custom_dash_params_uses_entity_values() {
    // Edge case: DashSpeedMultiplier(2.0), DashDuration(0.1) at position 0.0 dashing right
    //            teleports to 100.0 (500.0 * 2.0 * 0.1)
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: 0.35,
                ease_start: 0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(0.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            MaxSpeed(500.0),
            BreakerDeceleration(config.deceleration),
            DecelEasing {
                ease: config.decel_ease,
                strength: config.decel_ease_strength,
            },
            DashSpeedMultiplier(2.0),
            DashDuration(0.1),
            DashTilt(config.dash_tilt_angle.to_radians()),
        ))
        .insert((
            DashTiltEase(config.dash_tilt_ease),
            BrakeTilt {
                angle: config.brake_tilt_angle.to_radians(),
                duration: config.brake_tilt_duration,
                ease: config.brake_tilt_ease,
            },
            BrakeDecel(config.brake_decel_multiplier),
            SettleDuration(config.settle_duration),
            SettleTiltEase(config.settle_tilt_ease),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON,
        "teleport distance should use entity's DashSpeedMultiplier(2.0) and DashDuration(0.1): \
         expected 100.0 (500*2*0.1), got {}",
        pos.0.x
    );
}

// -- Behavior 3: Same-direction dash does normal dash ---------------

#[test]
fn same_direction_dash_with_flash_step_does_normal_dash() {
    // Given: Settling, last dash rightward (ease_start=-0.35), FlashStepActive
    // When: DashRight (same direction as last dash)
    // Then: BreakerState == Dashing (normal), velocity.x == 2000, position unchanged
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(0.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Dashing,
        "same-direction dash should do normal Dashing, not teleport"
    );

    let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
    assert!(
        (vel.x - 2000.0).abs() < f32::EPSILON,
        "same-direction dash should set normal velocity 2000 (500*4), got {}",
        vel.x
    );

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "same-direction dash should NOT teleport, position should be 0.0, got {}",
        pos.0.x
    );
}

#[test]
fn same_direction_dash_leftward_settle_with_dash_left_does_normal_dash() {
    // Edge case: leftward settle tilt (ease_start=0.35) + DashLeft = same direction
    let mut app = test_app();
    let entity = spawn_settling_breaker_leftward_dash(&mut app, Vec2::new(0.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Dashing,
        "leftward settle + DashLeft = same direction, should do normal Dashing"
    );
}

// -- Behavior 4: No FlashStepActive does normal dash ----------------

#[test]
fn reversal_dash_without_flash_step_does_normal_dash() {
    // Given: Settling, last dash rightward, NO FlashStepActive
    // When: DashLeft (reversal direction)
    // Then: BreakerState == Dashing, velocity.x == -2000, position unchanged
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(0.0, -250.0), false);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Dashing,
        "without FlashStepActive, reversal dash should do normal Dashing"
    );

    let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
    assert!(
        (vel.x - (-2000.0)).abs() < f32::EPSILON,
        "normal dash left velocity should be -2000, got {}",
        vel.x
    );

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "normal dash should NOT teleport, position should be 0.0, got {}",
        pos.0.x
    );
}

// -- Behavior 5: Idle with FlashStep does normal dash ---------------

#[test]
fn dash_from_idle_with_flash_step_does_normal_dash() {
    // Given: Idle state, FlashStepActive present
    // When: DashLeft
    // Then: BreakerState == Dashing, velocity.x == -2000, position unchanged
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Idle,
            BreakerVelocity { x: 0.0 },
            BreakerTilt::default(),
            BreakerStateTimer { remaining: 0.0 },
            Position2D(Vec2::new(0.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(
        *state,
        BreakerState::Dashing,
        "from Idle with FlashStep, dash should still be normal Dashing"
    );

    let vel = app.world().get::<BreakerVelocity>(entity).unwrap();
    assert!(
        (vel.x - (-2000.0)).abs() < f32::EPSILON,
        "from Idle, dash velocity should be normal -2000, got {}",
        vel.x
    );

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "from Idle, dash should NOT teleport, position should be 0.0, got {}",
        pos.0.x
    );
}
