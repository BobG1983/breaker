use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    breaker::{
        components::{BaseWidth, Breaker, BreakerTilt, DashState, DashStateTimer},
        definition::BreakerDefinition,
    },
    effect::effects::{flash_step::FlashStepActive, size_boost::ActiveSizeBoosts},
    input::resources::{GameAction, InputActions},
};

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

// -- Behavior 6: Teleport clamps to playfield right boundary --------

#[test]
fn flash_step_teleport_clamps_to_playfield_right_boundary() {
    // Given: Breaker at (350, -250), Settling from leftward dash (ease_start=0.35),
    //        FlashStepActive, BaseWidth(120), playfield right=400
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

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle);
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
    //        FlashStepActive, BaseWidth(120), playfield left=-400
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

    let state = app.world().get::<DashState>(entity).unwrap();
    assert_eq!(*state, DashState::Idle);
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
    //        BaseWidth(120), ActiveSizeBoosts(vec![2.0]), playfield right=400
    // When: DashRight
    // Then: Position2D.x == 280 (400 - 120 effective half-width from 60*2.0), NOT 600
    let mut app = test_app();
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle: 0.35,
                ease_start: 0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(300.0, -250.0)),
            BaseWidth(120.0),
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
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle: 0.35,
                ease_start: 0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(350.0, -250.0)),
            BaseWidth(120.0),
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
