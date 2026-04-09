use bevy::prelude::*;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Velocity2D};

use super::system::*;
use crate::{
    breaker::{
        components::{
            BaseWidth, Breaker, BreakerAcceleration, BreakerDeceleration, DashState, DecelEasing,
        },
        definition::BreakerDefinition,
    },
    effect::effects::{size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts},
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

#[test]
fn deceleration_toward_zero_positive() {
    let mut vel = 100.0;
    apply_deceleration(&mut vel, 500.0, 0.1);
    assert!((vel - 50.0).abs() < f32::EPSILON);
}

#[test]
fn deceleration_toward_zero_negative() {
    let mut vel = -100.0;
    apply_deceleration(&mut vel, 500.0, 0.1);
    assert!((vel - (-50.0)).abs() < f32::EPSILON);
}

#[test]
fn deceleration_clamps_at_zero() {
    let mut vel = 10.0;
    apply_deceleration(&mut vel, 500.0, 1.0);
    assert!((vel - 0.0).abs() < f32::EPSILON);
}

fn integration_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<InputActions>()
        .add_systems(FixedUpdate, move_breaker);
    app
}

use crate::shared::test_utils::tick;

fn spawn_breaker_at(app: &mut App, state: DashState, position: Vec2) -> Entity {
    let def = BreakerDefinition::default();
    app.world_mut()
        .spawn((
            Breaker,
            state,
            Velocity2D(Vec2::ZERO),
            MaxSpeed(def.max_speed),
            BreakerAcceleration(def.acceleration),
            BreakerDeceleration(def.deceleration),
            DecelEasing {
                ease: def.decel_ease,
                strength: def.decel_ease_strength,
            },
            BaseWidth(def.width),
            Position2D(position),
        ))
        .id()
}

fn spawn_breaker(app: &mut App, state: DashState) -> Entity {
    let def = BreakerDefinition::default();
    spawn_breaker_at(app, state, Vec2::new(0.0, def.y_position))
}

#[test]
fn right_input_moves_breaker_right() {
    // Given: Breaker in Idle state, Velocity2D(Vec2::ZERO),
    //        Position2D(Vec2::new(0.0, -250.0)), dt=1/60
    // When: move_breaker runs with MoveRight input
    // Then: Position2D.0.x > 0.0
    let mut app = integration_app();
    let entity = spawn_breaker(&mut app, DashState::Idle);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::MoveRight);
    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        pos.0.x > 0.0,
        "breaker should move right, got Position2D.x={}",
        pos.0.x
    );
}

#[test]
fn dashing_blocks_input_acceleration() {
    let mut app = integration_app();
    let entity = spawn_breaker(&mut app, DashState::Dashing);

    // Set velocity to zero, then push MoveRight — Dashing should not accelerate
    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::MoveRight);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.0.x.abs() < f32::EPSILON,
        "dashing state should not accelerate from keyboard, got vx={}",
        vel.0.x
    );
}

#[test]
fn position_clamped_to_playfield_bounds() {
    // Given: Breaker Position2D(Vec2::new(9999.0, -250.0)), BaseWidth(120.0),
    //        playfield right=400
    // When: move_breaker runs
    // Then: Position2D.0.x <= 340.0 (400 - 60)
    let mut app = integration_app();
    let def = BreakerDefinition::default();
    let entity = spawn_breaker_at(&mut app, DashState::Idle, Vec2::new(9999.0, def.y_position));
    let playfield = app.world().resource::<PlayfieldConfig>().clone();
    let half_width = app.world().get::<BaseWidth>(entity).unwrap().half_width();

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let max_x = playfield.right() - half_width;
    assert!(
        pos.0.x <= max_x + f32::EPSILON,
        "breaker should be clamped to playfield, got Position2D.x={} max={}",
        pos.0.x,
        max_x
    );
}

#[test]
fn speed_multiplier_raises_effective_max_speed() {
    // Given: MaxSpeed(500.0) + ActiveSpeedBoosts(vec![1.2]), velocity.x = 590.0
    //        MoveRight input active (so the acceleration+clamp path runs)
    // When: move_breaker system runs
    // Then: velocity.x > 500 AND velocity.x <= 600 (effective max = 500 * 1.2 = 600)
    let mut app = integration_app();
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::new(590.0, 0.0)),
            MaxSpeed(500.0),
            BreakerAcceleration(def.acceleration),
            BreakerDeceleration(def.deceleration),
            DecelEasing {
                ease: def.decel_ease,
                strength: def.decel_ease_strength,
            },
            BaseWidth(def.width),
            ActiveSpeedBoosts(vec![1.2]),
            Position2D(Vec2::new(0.0, def.y_position)),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::MoveRight);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.0.x > 500.0 + f32::EPSILON,
        "velocity {:.3} should NOT be clamped to base 500 when ActiveSpeedBoosts([1.2]) makes effective max 600",
        vel.0.x
    );
    assert!(
        vel.0.x <= 600.0 + f32::EPSILON,
        "velocity {:.3} should be clamped to effective max 600.0 (500 * 1.2)",
        vel.0.x
    );
}

#[test]
fn no_speed_boost_base_max_speed_clamps_velocity() {
    // Regression guard: without BreakerSpeedBoost, velocity above max_speed IS clamped.
    let mut app = integration_app();
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::new(600.0, 0.0)),
            MaxSpeed(500.0),
            BreakerAcceleration(0.0),
            BreakerDeceleration(0.0),
            DecelEasing {
                ease: def.decel_ease,
                strength: def.decel_ease_strength,
            },
            BaseWidth(def.width),
            Position2D(Vec2::new(0.0, def.y_position)),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::MoveRight);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.0.x <= 500.0 + f32::EPSILON,
        "velocity {:.3} should be clamped to base max_speed 500.0 with no boost",
        vel.0.x
    );
}

#[test]
fn size_multiplier_increases_effective_half_width_for_clamping() {
    // Given: BaseWidth(120.0), ActiveSizeBoosts(vec![4/3]), PlayfieldConfig default (right=400)
    //        effective half_w = 60.0 * (4/3) = 80
    //        Breaker placed far right (9999.0) — position will be clamped during tick
    // When: move_breaker runs
    // Then: Position2D.0.x clamped to max_x = 400 - 80 = 320
    let mut app = integration_app();
    let def = BreakerDefinition::default();

    let playfield = app.world().resource::<PlayfieldConfig>().clone();
    assert!(
        (playfield.right() - 400.0).abs() < 1.0,
        "test assumes default playfield right=400, got {}",
        playfield.right()
    );

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::ZERO),
            MaxSpeed(def.max_speed),
            BreakerAcceleration(0.0),
            BreakerDeceleration(0.0),
            DecelEasing {
                ease: def.decel_ease,
                strength: def.decel_ease_strength,
            },
            BaseWidth(120.0),
            ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
            Position2D(Vec2::new(9999.0, def.y_position)),
        ))
        .id();

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let expected_max_x = 320.0_f32;
    assert!(
        pos.0.x <= expected_max_x + f32::EPSILON,
        "with ActiveSizeBoosts([4/3]) effective half_w=80, Position2D.x {:.3} should be clamped to {:.3}, not to base {:.3}",
        pos.0.x,
        expected_max_x,
        400.0 - 60.0
    );
}

#[test]
fn move_breaker_reads_active_speed_boosts_for_max_speed() {
    // Given: Breaker with ActiveSpeedBoosts(vec![2.0]), MaxSpeed(300.0),
    //        Velocity2D(Vec2::new(590.0, 0.0)), MoveRight active
    // When: move_breaker runs
    // Then: velocity.x clamped to 600.0 (300.0 * 2.0), not 300.0
    let mut app = integration_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::new(590.0, 0.0)),
            MaxSpeed(300.0),
            BreakerAcceleration(5000.0),
            BreakerDeceleration(3000.0),
            DecelEasing {
                ease: bevy::math::curve::easing::EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
            ActiveSpeedBoosts(vec![2.0]),
            Position2D(Vec2::new(0.0, -250.0)),
        ))
        .id();

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::MoveRight);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.0.x > 300.0 + f32::EPSILON,
        "velocity {:.3} should exceed base max 300.0 with ActiveSpeedBoosts([2.0]) -> effective max 600.0",
        vel.0.x
    );
    assert!(
        vel.0.x <= 600.0 + f32::EPSILON,
        "velocity {:.3} should be clamped to effective max 600.0 (300 * 2.0)",
        vel.0.x
    );
}

#[test]
fn move_breaker_reads_active_size_boosts_for_playfield_clamping() {
    // Given: Breaker with ActiveSizeBoosts(vec![2.0]), BaseWidth(120.0) (half_width=60.0),
    //        Position2D far right (9999.0)
    // When: move_breaker clamps position
    // Then: position clamped using effective_half_w = 60.0 * 2.0 = 120.0
    //       -> max_x = 400.0 - 120.0 = 280.0
    let mut app = integration_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            DashState::Idle,
            Velocity2D(Vec2::ZERO),
            MaxSpeed(300.0),
            BreakerAcceleration(0.0),
            BreakerDeceleration(0.0),
            DecelEasing {
                ease: bevy::math::curve::easing::EaseFunction::QuadraticIn,
                strength: 1.0,
            },
            BaseWidth(120.0),
            ActiveSizeBoosts(vec![2.0]),
            Position2D(Vec2::new(9999.0, -250.0)),
        ))
        .id();

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    let expected_max_x = 280.0_f32; // 400.0 - (60.0 * 2.0)
    assert!(
        pos.0.x <= expected_max_x + f32::EPSILON,
        "with ActiveSizeBoosts([2.0]) effective half_w=120.0, Position2D.x {:.3} should be clamped to {:.3}",
        pos.0.x,
        expected_max_x
    );
}
