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
    effect::EffectiveSpeedMultiplier,
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

// --- EffectiveSpeedMultiplier tests ---

/// Spec behavior 5: `EffectiveSpeedMultiplier` raises the effective minimum speed multiplicatively.
///
/// Given: speed=100, min=200, max=600, `EffectiveSpeedMultiplier(1.5)` -> `effective_min`=300.
/// Speed 100 < 300 -> should clamp UP to 300.
#[test]
fn effective_speed_multiplier_raises_effective_min_speed() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 100.0)), // speed=100
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
            EffectiveSpeedMultiplier(1.5), // effective_min = 200 * 1.5 = 300
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.speed() >= 300.0 - f32::EPSILON,
        "speed {} should be at least effective_min 300.0 (base 200 * 1.5)",
        vel.speed()
    );
}

/// Spec behavior 6: `EffectiveSpeedMultiplier` clamps high speed to boosted max.
///
/// Given: speed=1000, min=200, max=600, `EffectiveSpeedMultiplier(1.5)` -> `effective_max`=900.
/// Speed 1000 > 900 -> should clamp DOWN to 900.
#[test]
fn effective_speed_multiplier_clamps_high_speed_to_boosted_max() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 1000.0)), // speed=1000
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
            EffectiveSpeedMultiplier(1.5), // effective_max = 600 * 1.5 = 900
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 900.0).abs() < 1.0,
        "speed {} should be clamped down to effective_max 900.0 (base 600 * 1.5)",
        vel.speed()
    );
}

/// Spec behavior 7: No `EffectiveSpeedMultiplier` uses base min/max.
///
/// Given: speed=100, min=200, max=600, NO `EffectiveSpeedMultiplier`.
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
            // No EffectiveSpeedMultiplier
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 200.0).abs() < 1.0,
        "speed {} should be clamped to base min 200.0 when no EffectiveSpeedMultiplier present",
        vel.speed()
    );
}

/// [`BoltServing`] bolt is not affected by [`EffectiveSpeedMultiplier`] (excluded by `ActiveFilter`).
///
/// Given: serving bolt, speed=1, min=200, max=600, `EffectiveSpeedMultiplier(1.5)`.
/// `ActiveFilter` excludes [`BoltServing`] -> velocity unchanged at speed=1.
#[test]
fn serving_bolt_not_affected_by_effective_speed_multiplier() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BoltServing,
            Velocity2D(Vec2::new(0.0, 1.0)), // speed=1, below any min
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
            EffectiveSpeedMultiplier(1.5), // effective_min would be 300
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 1.0).abs() < f32::EPSILON,
        "serving bolt speed {} should be unchanged at 1.0 (excluded by ActiveFilter)",
        vel.speed()
    );
}

/// `EffectiveSpeedMultiplier(1.0)` is identical to no multiplier -- base clamping applies.
///
/// Given: speed=600, min=200, max=600, `EffectiveSpeedMultiplier(1.0)` -> `effective_max`=600.
/// Speed 600 == `effective_max` -> should remain at 600 (no change needed).
#[test]
fn effective_speed_multiplier_identity_same_as_no_multiplier() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)), // speed=600, exactly at base max
            BoltMinSpeed(200.0),
            BoltMaxSpeed(600.0),
            EffectiveSpeedMultiplier(1.0), // identity -- no change expected
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 600.0).abs() < 1.0,
        "speed {} should remain at 600.0 when EffectiveSpeedMultiplier is 1.0 (identity)",
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
