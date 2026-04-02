use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    breaker::{
        components::{BaseWidth, Breaker, BreakerTilt, DashState, DashStateTimer},
        definition::BreakerDefinition,
    },
    effect::effects::{
        flash_step::FlashStepActive, size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
    },
    input::resources::{GameAction, InputActions},
};

// -- Behavior 9: ActiveSpeedBoosts affects teleport distance --

#[test]
fn flash_step_teleport_respects_speed_multiplier_for_distance() {
    // Given: Breaker at (200, -250), Settling from rightward dash (ease_start=-0.35),
    //        FlashStepActive, ActiveSpeedBoosts(vec![1.5]), MaxSpeed(500),
    //        DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashLeft
    // Then: Position2D.x == -250.0 (200 + (-1)*500*1.5*4*0.15 = 200 - 450)
    let mut app = test_app();
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(200.0, -250.0)),
            BaseWidth(120.0),
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
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
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

// -- Behavior 4: ActiveSpeedBoosts affects flash step teleport distance --

#[test]
fn flash_step_teleport_reads_active_speed_boosts_for_distance() {
    // Given: Breaker at (200.0, -250.0), Settling from rightward dash (ease_start=-0.35),
    //        FlashStepActive, ActiveSpeedBoosts(vec![1.5]), MaxSpeed(500),
    //        DashSpeedMultiplier(4), DashDuration(0.15)
    // When: DashLeft
    // Then: Position2D.x = 200.0 + (-1) * 500.0 * 1.5 * 4.0 * 0.15 = 200.0 - 450.0 = -250.0
    let mut app = test_app();
    let config = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Settling,
            Velocity2D(Vec2::ZERO),
            BreakerTilt {
                angle: -0.35,
                ease_start: -0.35,
                ease_target: 0.0,
            },
            DashStateTimer { remaining: 0.2 },
            Position2D(Vec2::new(200.0, -250.0)),
            BaseWidth(120.0),
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
    //        FlashStepActive, ActiveSizeBoosts(vec![2.0]), BaseWidth(120.0) (half_width=60.0),
    //        DashRight input, playfield right = 400.0
    // When: dash system clamps after flash step teleport
    // Then: effective_half_w = 60.0 * 2.0 = 120.0 -> max_x = 400.0 - 120.0 = 280.0
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
    let expected_max_x = 280.0_f32; // 400.0 - (60.0 * 2.0)
    assert!(
        (pos.0.x - expected_max_x).abs() < f32::EPSILON,
        "with ActiveSizeBoosts([2.0]), clamp to {:.1} \
         (400 - 60*2.0), got {}",
        expected_max_x,
        pos.0.x
    );
}
