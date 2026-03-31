use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltServing},
        resources::BoltConfig,
    },
    breaker::{
        components::{Breaker, MinAngleFromHorizontal},
        resources::BreakerConfig,
    },
    effect::effects::speed_boost::ActiveSpeedBoosts,
};

fn bolt_param_bundle() -> (BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed) {
    let bolt_config = BoltConfig::default();
    (
        BoltBaseSpeed(bolt_config.base_speed),
        BoltMinSpeed(bolt_config.min_speed),
        BoltMaxSpeed(bolt_config.max_speed),
    )
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, prepare_bolt_velocity);
    // Spawn breaker with MinAngleFromHorizontal for the system to read
    let breaker_config = BreakerConfig::default();
    app.world_mut().spawn((
        Breaker,
        MinAngleFromHorizontal(breaker_config.min_angle_from_horizontal.to_radians()),
    ));
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

#[test]
fn move_bolt_does_not_translate_position() {
    let mut app = test_app();

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, 400.0)),
        bolt_param_bundle(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let tf = app
        .world_mut()
        .query_filtered::<&Transform, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    assert!(
        tf.translation.y.abs() < f32::EPSILON,
        "move_bolt should NOT update position (CCD handles that), got y={}",
        tf.translation.y
    );
}

#[test]
fn serving_bolt_velocity_unchanged() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 1.0)), // below min_speed
            bolt_param_bundle(),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 1.0).abs() < f32::EPSILON,
        "serving bolt velocity should not be clamped, got speed={}",
        vel.speed()
    );
}

#[test]
fn no_breaker_leaves_velocity_unchanged() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, prepare_bolt_velocity);
    // No breaker entity spawned

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 1.0)), // below min, but no breaker -> early return
            bolt_param_bundle(),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 1.0).abs() < f32::EPSILON,
        "without breaker, velocity should be unchanged, got speed={}",
        vel.speed()
    );
}

#[test]
fn speed_below_min_is_clamped_up() {
    let mut app = test_app();
    let config = BoltConfig::default();

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, 1.0)), // far below min_speed
        bolt_param_bundle(),
    ));

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(
        vel.speed() >= config.min_speed - f32::EPSILON,
        "speed {} should be at least min_speed {}",
        vel.speed(),
        config.min_speed
    );
}

/// No `EffectiveSpeedMultiplier` and no `ActiveSpeedBoosts` uses base min/max.
///
/// Given: speed=100, min=200, max=600, NO `EffectiveSpeedMultiplier`, NO `ActiveSpeedBoosts`.
/// Speed 100 < 200 -> clamped to 200 (base min). No multiplier applied.
#[test]
fn no_effective_speed_multiplier_uses_base_min_speed() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 100.0)), // speed=100
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
            // No EffectiveSpeedMultiplier, no ActiveSpeedBoosts
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 200.0).abs() < 1.0,
        "speed {} should be clamped to base min 200.0 when no speed multiplier present",
        vel.speed()
    );
}

// -- Velocity2D migration tests --

/// Given: bolt with `Velocity2D`(0.0, 100.0), min=200, max=600.
/// When: `prepare_bolt_velocity` runs.
/// Then: `Velocity2D` speed clamped up to >= 200.
#[test]
fn velocity2d_speed_below_min_is_clamped_up() {
    use rantzsoft_spatial2d::components::Velocity2D;

    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 100.0)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
        ))
        .id();

    tick(&mut app);

    let vel = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("entity should have Velocity2D");
    assert!(
        vel.speed() >= 200.0 - f32::EPSILON,
        "Velocity2D speed {} should be clamped up to at least min_speed 200.0",
        vel.speed()
    );
}

/// Given: bolt with `Velocity2D`(0.0, 800.0), min=200, max=600.
/// When: `prepare_bolt_velocity` runs.
/// Then: `Velocity2D` speed clamped down to 600.
#[test]
fn velocity2d_speed_above_max_is_clamped_down() {
    use rantzsoft_spatial2d::components::Velocity2D;

    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
        ))
        .id();

    tick(&mut app);

    let vel = app
        .world()
        .get::<Velocity2D>(entity)
        .expect("entity should have Velocity2D");
    assert!(
        (vel.speed() - 600.0).abs() < 1.0,
        "Velocity2D speed {} should be clamped down to max_speed 600.0",
        vel.speed()
    );
}

// -- Regression: speed clamp must use ActiveSpeedBoosts, not stale EffectiveSpeedMultiplier --

/// Speed clamp should use the current product of `ActiveSpeedBoosts` for its effective
/// min/max calculation.
///
/// Given: speed=400, min=200, max=800, `ActiveSpeedBoosts([2.0])` (product=2.0).
/// When: `prepare_bolt_velocity` runs.
/// Then: `effective_min` = 200 * 2.0 = 400, speed 400 is within [400, 1600]. Speed unchanged.
#[test]
fn speed_clamp_uses_current_active_speed_boosts() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
            ActiveSpeedBoosts(vec![2.0]),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    // With mult=2.0, effective_min=400, effective_max=1600. Speed 400 is exactly at effective_min.
    assert!(
        (vel.speed() - 400.0).abs() < 1.0,
        "speed {} should remain at 400.0 (within boosted range [400, 1600])",
        vel.speed()
    );
}

/// When `ActiveSpeedBoosts` is present but `EffectiveSpeedMultiplier` is absent, the system
/// should compute the multiplier from `ActiveSpeedBoosts` rather than defaulting to 1.0.
///
/// Given: speed=100, min=200, max=800, `ActiveSpeedBoosts([2.0])` (product=2.0),
///        NO `EffectiveSpeedMultiplier`.
/// When: `prepare_bolt_velocity` runs.
/// Then: Correct `effective_min` = 200 * 2.0 = 400. Speed 100 < 400, clamp UP to 400.
///
/// BUG: current implementation sees no `EffectiveSpeedMultiplier`, defaults to mult=1.0,
/// computes `effective_min` = 200. Clamps speed up to 200, not the correct 400.
#[test]
fn speed_clamp_without_effective_multiplier_uses_active_boosts() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 100.0)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
            ActiveSpeedBoosts(vec![2.0]), // product = 2.0
                                          // No EffectiveSpeedMultiplier — simulates component not yet inserted
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    // Correct behavior: mult from ActiveSpeedBoosts = 2.0, effective_min = 400.
    // Speed 100 < 400, should be clamped UP to 400.
    assert!(
        (vel.speed() - 400.0).abs() < 1.0,
        "speed {} should be clamped up to effective_min 400.0 (base 200 * ActiveSpeedBoosts product 2.0), \
         not 200.0 from default mult=1.0 when EffectiveSpeedMultiplier is absent",
        vel.speed()
    );
}
