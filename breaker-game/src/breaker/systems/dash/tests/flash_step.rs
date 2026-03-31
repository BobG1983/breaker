//! `FlashStep` teleport tests -- reversal dash during settling with `FlashStepActive`
//! teleports the breaker instantly instead of doing a normal dash.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::system::update_breaker_state;
use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerMaxSpeed, BreakerState,
            BreakerStateTimer, BreakerTilt, BreakerVelocity, BreakerWidth, DashDuration,
            DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, SettleDuration,
            SettleTiltEase,
        },
        resources::BreakerConfig,
    },
    effect::effects::{
        flash_step::FlashStepActive, size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
    },
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

// -- Helpers --------------------------------------------------------

fn breaker_param_bundle(
    config: &BreakerConfig,
) -> (
    BreakerMaxSpeed,
    BreakerDeceleration,
    DecelEasing,
    DashSpeedMultiplier,
    DashDuration,
    DashTilt,
    DashTiltEase,
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
        DashTilt(config.dash_tilt_angle.to_radians()),
        DashTiltEase(config.dash_tilt_ease),
        BrakeTilt {
            angle: config.brake_tilt_angle.to_radians(),
            duration: config.brake_tilt_duration,
            ease: config.brake_tilt_ease,
        },
        BrakeDecel(config.brake_decel_multiplier),
        SettleDuration(config.settle_duration),
        SettleTiltEase(config.settle_tilt_ease),
    )
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerConfig>()
        .init_resource::<InputActions>()
        .init_resource::<PlayfieldConfig>()
        .add_systems(FixedUpdate, update_breaker_state);
    app
}

/// Accumulates one fixed timestep of overstep, then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Spawns a breaker in Settling state with a rightward-dash settle tilt
/// (`ease_start` < 0, meaning last dash was rightward).
///
/// Returns the entity ID.
fn spawn_settling_breaker_rightward_dash(
    app: &mut App,
    position: Vec2,
    flash_step: bool,
) -> Entity {
    let config = BreakerConfig::default();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        BreakerState::Settling,
        BreakerVelocity { x: 0.0 },
        BreakerTilt {
            angle: -0.35,
            ease_start: -0.35,
            ease_target: 0.0,
        },
        BreakerStateTimer { remaining: 0.2 },
        Position2D(position),
        BreakerWidth(120.0),
        breaker_param_bundle(&config),
    ));
    if flash_step {
        entity_cmds.insert(FlashStepActive);
    }
    entity_cmds.id()
}

/// Spawns a breaker in Settling state with a leftward-dash settle tilt
/// (`ease_start` > 0, meaning last dash was leftward).
///
/// Returns the entity ID.
fn spawn_settling_breaker_leftward_dash(app: &mut App, position: Vec2, flash_step: bool) -> Entity {
    let config = BreakerConfig::default();
    let mut entity_cmds = app.world_mut().spawn((
        Breaker,
        BreakerState::Settling,
        BreakerVelocity { x: 0.0 },
        BreakerTilt {
            angle: 0.35,
            ease_start: 0.35,
            ease_target: 0.0,
        },
        BreakerStateTimer { remaining: 0.2 },
        Position2D(position),
        BreakerWidth(120.0),
        breaker_param_bundle(&config),
    ));
    if flash_step {
        entity_cmds.insert(FlashStepActive);
    }
    entity_cmds.id()
}

// -- Behavior 1: Reversal dash right-to-left teleports breaker ------

#[test]
fn reversal_dash_left_during_settling_with_flash_step_teleports_to_endpoint() {
    // Given: Breaker in Settling at (0.0, -250.0), last dash rightward (ease_start=-0.35),
    //        FlashStepActive, BreakerMaxSpeed(500), DashSpeedMultiplier(4), DashDuration(0.15)
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

#[test]
fn reversal_dash_left_clamps_to_playfield_left_bound() {
    // Edge case: Breaker at (-350, -250) dashing left with 300 teleport distance
    //            would go to -650 but clamps to -400 + 60 = -340
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(-350.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-340.0)).abs() < f32::EPSILON,
        "teleport should clamp to left bound -340.0 (-400 + 60 half-width), got {}",
        pos.0.x
    );
}

// -- Behavior 2: Reversal dash left-to-right also teleports ---------

#[test]
fn reversal_dash_right_during_settling_with_flash_step_teleports_to_endpoint() {
    // Given: Breaker in Settling at (0.0, -250.0), last dash leftward (ease_start=0.35),
    //        FlashStepActive, BreakerMaxSpeed(500), DashSpeedMultiplier(4), DashDuration(0.15)
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
            BreakerMaxSpeed(500.0),
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

// -- Behavior 6: Teleport clamps to playfield right boundary --------

#[test]
fn flash_step_teleport_clamps_to_playfield_right_boundary() {
    // Given: Breaker at (350, -250), Settling from leftward dash (ease_start=0.35),
    //        FlashStepActive, BreakerWidth(120), playfield right=400
    // When: DashRight
    // Then: Position2D.x == 340 (400 - 60 half-width), NOT 650
    let mut app = test_app();
    let entity = spawn_settling_breaker_leftward_dash(&mut app, Vec2::new(350.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 340.0).abs() < f32::EPSILON,
        "teleport should clamp to right bound 340.0 (400 - 60), got {}",
        pos.0.x
    );

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(*state, BreakerState::Idle);
}

#[test]
fn flash_step_teleport_at_right_boundary_stays_at_boundary() {
    // Edge case: already at right boundary (340.0) teleporting right stays at 340.0
    let mut app = test_app();
    let entity = spawn_settling_breaker_leftward_dash(&mut app, Vec2::new(340.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 340.0).abs() < f32::EPSILON,
        "at right boundary, teleporting right should stay at 340.0, got {}",
        pos.0.x
    );
}

// -- Behavior 7: Teleport clamps to playfield left boundary ---------

#[test]
fn flash_step_teleport_clamps_to_playfield_left_boundary() {
    // Given: Breaker at (-350, -250), Settling from rightward dash (ease_start=-0.35),
    //        FlashStepActive, BreakerWidth(120), playfield left=-400
    // When: DashLeft
    // Then: Position2D.x == -340 (-400 + 60 half-width), NOT -650
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(-350.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-340.0)).abs() < f32::EPSILON,
        "teleport should clamp to left bound -340.0 (-400 + 60), got {}",
        pos.0.x
    );

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(*state, BreakerState::Idle);
}

#[test]
fn flash_step_teleport_at_left_boundary_stays_at_boundary() {
    // Edge case: already at left boundary (-340.0) teleporting left stays at -340.0
    let mut app = test_app();
    let entity = spawn_settling_breaker_rightward_dash(&mut app, Vec2::new(-340.0, -250.0), true);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-340.0)).abs() < f32::EPSILON,
        "at left boundary, teleporting left should stay at -340.0, got {}",
        pos.0.x
    );
}

// -- Behavior 8: ActiveSizeBoosts adjusts clamping -----------

#[test]
fn flash_step_teleport_with_size_multiplier_adjusts_clamp_half_width() {
    // Given: Breaker at (300, -250), Settling from leftward dash, FlashStepActive,
    //        BreakerWidth(120), ActiveSizeBoosts(vec![2.0]), playfield right=400
    // When: DashRight
    // Then: Position2D.x == 280 (400 - 120 effective half-width from 60*2.0), NOT 600
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
            Position2D(Vec2::new(300.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSizeBoosts(vec![2.0]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 280.0).abs() < f32::EPSILON,
        "with ActiveSizeBoosts([2.0]), clamp to 280.0 (400 - 60*2.0), got {}",
        pos.0.x
    );
}

#[test]
fn flash_step_teleport_with_size_multiplier_one_matches_no_multiplier() {
    // Edge case: ActiveSizeBoosts(vec![1.0]) behaves same as no multiplier
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
            Position2D(Vec2::new(350.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSizeBoosts(vec![1.0]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    // With ActiveSizeBoosts([1.0]), half_width = 60, so clamp = 400 - 60 = 340
    assert!(
        (pos.0.x - 340.0).abs() < f32::EPSILON,
        "ActiveSizeBoosts([1.0]) should clamp same as default: 340.0, got {}",
        pos.0.x
    );
}

// -- Behavior 9: ActiveSpeedBoosts affects teleport distance --

#[test]
fn flash_step_teleport_respects_speed_multiplier_for_distance() {
    // Given: Breaker at (200, -250), Settling from rightward dash (ease_start=-0.35),
    //        FlashStepActive, ActiveSpeedBoosts(vec![1.5]), BreakerMaxSpeed(500),
    //        DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashLeft
    // Then: Position2D.x == -250.0 (200 + (-1)*500*1.5*4*0.15 = 200 - 450)
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(200.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSpeedBoosts(vec![1.5]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-250.0)).abs() < 0.01,
        "with ActiveSpeedBoosts([1.5]), teleport to -250.0 (200 - 500*1.5*4*0.15=450), got {}",
        pos.0.x
    );
}

#[test]
fn flash_step_teleport_with_speed_multiplier_one_matches_no_multiplier() {
    // Edge case: ActiveSpeedBoosts(vec![1.0]) same result as no multiplier (300 distance)
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(0.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSpeedBoosts(vec![1.0]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-300.0)).abs() < f32::EPSILON,
        "ActiveSpeedBoosts([1.0]) should give same 300.0 distance, expected -300.0, got {}",
        pos.0.x
    );
}

// -- Behavior 10: Teleport resets tilt and timer cleanly to Idle ----

#[test]
fn flash_step_teleport_resets_tilt_and_timer_to_idle() {
    // Given: Settling with partially eased tilt (angle=-0.25, ease_start=-0.35, ease_target=0.0),
    //        timer remaining=0.15, FlashStepActive, reversal dash input
    // When: update_breaker_state runs
    // Then: tilt.angle == 0.0, tilt.ease_start == 0.0, tilt.ease_target == 0.0,
    //       timer.remaining == 0.0, BreakerState == Idle
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: -0.25,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.15 },
            Position2D(Vec2::new(0.0, -250.0)),
            BreakerWidth(120.0),
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

    let state = app.world().get::<BreakerState>(entity).unwrap();
    assert_eq!(*state, BreakerState::Idle, "teleport should reset to Idle");

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

    let timer = app.world().get::<BreakerStateTimer>(entity).unwrap();
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
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: -0.25,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.001 },
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
        BreakerState::Idle,
        "nearly expired timer should still produce clean Idle reset"
    );

    let tilt = app.world().get::<BreakerTilt>(entity).unwrap();
    assert!(
        tilt.angle.abs() < f32::EPSILON,
        "tilt.angle should be 0.0 even with nearly expired timer, got {}",
        tilt.angle
    );

    let timer = app.world().get::<BreakerStateTimer>(entity).unwrap();
    assert!(
        timer.remaining.abs() < f32::EPSILON,
        "timer.remaining should be 0.0, got {}",
        timer.remaining
    );
}

// -- Behavior 4: ActiveSpeedBoosts affects flash step teleport distance --

#[test]
fn flash_step_teleport_reads_active_speed_boosts_for_distance() {
    // Given: Breaker at (200.0, -250.0), Settling from rightward dash (ease_start=-0.35),
    //        FlashStepActive, ActiveSpeedBoosts(vec![1.5]), BreakerMaxSpeed(500),
    //        DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashLeft
    // Then: Position2D.x = 200.0 + (-1) * 500.0 * 1.5 * 4.0 * 0.15 = 200.0 - 450.0 = -250.0
    let mut app = test_app();
    let config = BreakerConfig::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerState::Settling,
            BreakerVelocity { x: 0.0 },
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(200.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSpeedBoosts(vec![1.5]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashLeft);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-250.0)).abs() < 0.01,
        "with ActiveSpeedBoosts([1.5]), teleport to -250.0 \
         (200 - 500*1.5*4*0.15=450), got {}",
        pos.0.x
    );
}

// -- Behavior 5: ActiveSizeBoosts affects flash step clamp half-width --

#[test]
fn flash_step_teleport_reads_active_size_boosts_for_clamp_half_width() {
    // Given: Breaker at (300.0, -250.0), Settling from leftward dash (ease_start=0.35),
    //        FlashStepActive, ActiveSizeBoosts(vec![2.0]), BreakerWidth(120.0) (half_width=60.0),
    //        DashRight input, playfield right = 400.0
    // When: dash system clamps after flash step teleport
    // Then: effective_half_w = 60.0 * 2.0 = 120.0 -> max_x = 400.0 - 120.0 = 280.0
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
            Position2D(Vec2::new(300.0, -250.0)),
            BreakerWidth(120.0),
            FlashStepActive,
            ActiveSizeBoosts(vec![2.0]),
            breaker_param_bundle(&config),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::DashRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let expected_max_x = 280.0_f32; // 400.0 - (60.0 * 2.0)
    assert!(
        (pos.0.x - expected_max_x).abs() < f32::EPSILON,
        "with ActiveSizeBoosts([2.0]), clamp to {:.1} \
         (400 - 60*2.0), got {}",
        expected_max_x,
        pos.0.x
    );
}
