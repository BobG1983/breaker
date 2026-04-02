use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use super::helpers::test_breaker_definition;
use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerInitialized, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow, BumpFeedback,
        BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown,
        DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
        DecelEasing, PrimaryBreaker, SettleDuration, SettleTiltEase,
    },
    effect::effects::life_lost::LivesCount,
    shared::{
        BaseHeight, BaseWidth, CleanupOnRunEnd, GameDrawLayer,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

// ── Behavior 28: build() on a headless primary breaker produces all core components ──

#[test]
fn build_headless_primary_has_breaker_marker() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Breaker>(entity).is_some(),
        "should have Breaker"
    );
    // Guard: check a non-#[require] component to prevent false pass from stub
    assert!(
        world.get::<BreakerInitialized>(entity).is_some(),
        "should have BreakerInitialized"
    );
    assert!(
        world.get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker"
    );
    assert!(
        world.get::<CleanupOnRunEnd>(entity).is_some(),
        "should have CleanupOnRunEnd"
    );
}

#[test]
fn build_headless_primary_has_default_state_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let vel = world.get::<Velocity2D>(entity);
    assert!(vel.is_some(), "should have Velocity2D");
    assert_eq!(vel.unwrap().0, Vec2::ZERO, "Velocity2D should be zero");

    let dash = world.get::<DashState>(entity);
    assert!(dash.is_some(), "should have DashState");
    assert_eq!(*dash.unwrap(), DashState::Idle, "DashState should be Idle");

    let tilt = world.get::<BreakerTilt>(entity);
    assert!(tilt.is_some(), "should have BreakerTilt");
    assert!(
        (tilt.unwrap().angle).abs() < f32::EPSILON,
        "BreakerTilt.angle should be 0.0"
    );

    let bump = world.get::<BumpState>(entity);
    assert!(bump.is_some(), "should have BumpState");
    assert!(!bump.unwrap().active, "BumpState.active should be false");

    let timer = world.get::<DashStateTimer>(entity);
    assert!(timer.is_some(), "should have DashStateTimer");
    assert!(
        (timer.unwrap().remaining).abs() < f32::EPSILON,
        "DashStateTimer.remaining should be 0.0"
    );

    let layer = world.get::<GameDrawLayer>(entity);
    assert!(layer.is_some(), "should have GameDrawLayer");
    assert!(
        matches!(layer.unwrap(), GameDrawLayer::Breaker),
        "GameDrawLayer should be Breaker"
    );

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "should have LivesCount");
}

// ── Behavior 29: build() produces correct spatial components ──

#[test]
fn build_produces_correct_spatial_components() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0, y_position: -250.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity);
    assert!(pos.is_some(), "should have Position2D");
    let pos = pos.unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "Position2D.x should be 0.0"
    );
    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "Position2D.y should be -250.0"
    );

    let prev = world.get::<PreviousPosition>(entity);
    assert!(prev.is_some(), "should have PreviousPosition");
    let prev = prev.unwrap();
    assert!(
        (prev.0.x - 0.0).abs() < f32::EPSILON,
        "PreviousPosition.x should be 0.0"
    );
    assert!(
        (prev.0.y - (-250.0)).abs() < f32::EPSILON,
        "PreviousPosition.y should be -250.0"
    );

    let scale = world.get::<Scale2D>(entity);
    assert!(scale.is_some(), "should have Scale2D");
    assert!(
        (scale.unwrap().x - 120.0).abs() < f32::EPSILON,
        "Scale2D.x should be 120.0"
    );
    assert!(
        (scale.unwrap().y - 20.0).abs() < f32::EPSILON,
        "Scale2D.y should be 20.0"
    );

    let prev_scale = world.get::<PreviousScale>(entity);
    assert!(prev_scale.is_some(), "should have PreviousScale");
    assert!((prev_scale.unwrap().x - 120.0).abs() < f32::EPSILON);
    assert!((prev_scale.unwrap().y - 20.0).abs() < f32::EPSILON);

    let aabb = world.get::<Aabb2D>(entity);
    assert!(aabb.is_some(), "should have Aabb2D");
    assert!((aabb.unwrap().half_extents.x - 60.0).abs() < f32::EPSILON);
    assert!((aabb.unwrap().half_extents.y - 10.0).abs() < f32::EPSILON);
}

// ── Behavior 30: build() produces correct physics components ──

#[test]
fn build_produces_correct_collision_layers() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let layers = world.get::<CollisionLayers>(entity);
    assert!(layers.is_some(), "should have CollisionLayers");
    let layers = layers.unwrap();
    assert_eq!(
        layers.membership,
        crate::shared::BREAKER_LAYER,
        "membership should be BREAKER_LAYER"
    );
    assert_eq!(
        layers.mask,
        crate::shared::BOLT_LAYER,
        "mask should be BOLT_LAYER"
    );
}

// ── Behavior 31: build() produces correct dimension components ──

#[test]
fn build_produces_correct_dimension_components() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0, y_position: -250.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    assert!((world.get::<BaseWidth>(entity).unwrap().0 - 120.0).abs() < f32::EPSILON);
    assert!((world.get::<BaseHeight>(entity).unwrap().0 - 20.0).abs() < f32::EPSILON);
    assert!((world.get::<MinWidth>(entity).unwrap().0 - 60.0).abs() < f32::EPSILON);
    assert!((world.get::<MaxWidth>(entity).unwrap().0 - 600.0).abs() < f32::EPSILON);
    assert!((world.get::<MinHeight>(entity).unwrap().0 - 10.0).abs() < f32::EPSILON);
    assert!((world.get::<MaxHeight>(entity).unwrap().0 - 100.0).abs() < f32::EPSILON);
    assert!((world.get::<BreakerBaseY>(entity).unwrap().0 - (-250.0)).abs() < f32::EPSILON);
}

// ── Behavior 32: build() produces correct movement components ──

#[test]
fn build_produces_correct_movement_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    assert!((world.get::<MaxSpeed>(entity).unwrap().0 - 500.0).abs() < f32::EPSILON);
    assert!((world.get::<BreakerAcceleration>(entity).unwrap().0 - 3000.0).abs() < f32::EPSILON);
    assert!((world.get::<BreakerDeceleration>(entity).unwrap().0 - 2500.0).abs() < f32::EPSILON);
    let de = world.get::<DecelEasing>(entity).unwrap();
    assert!(matches!(de.ease, EaseFunction::QuadraticIn));
    assert!((de.strength - 1.0).abs() < f32::EPSILON);
}

// ── Behavior 33: build() produces correct dash components ──

#[test]
fn build_produces_correct_dash_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    assert!((world.get::<DashSpeedMultiplier>(entity).unwrap().0 - 4.0).abs() < f32::EPSILON);
    assert!((world.get::<DashDuration>(entity).unwrap().0 - 0.15).abs() < f32::EPSILON);
    assert!((world.get::<DashTilt>(entity).unwrap().0 - 15.0_f32.to_radians()).abs() < 1e-5);
    assert!(matches!(
        world.get::<DashTiltEase>(entity).unwrap().0,
        EaseFunction::QuadraticInOut
    ));
    let bt = world.get::<BrakeTilt>(entity).unwrap();
    assert!((bt.angle - 25.0_f32.to_radians()).abs() < 1e-5);
    assert!((bt.duration - 0.2).abs() < f32::EPSILON);
    assert!(matches!(bt.ease, EaseFunction::CubicInOut));
    assert!((world.get::<BrakeDecel>(entity).unwrap().0 - 2.0).abs() < f32::EPSILON);
    assert!((world.get::<SettleDuration>(entity).unwrap().0 - 0.25).abs() < f32::EPSILON);
    assert!(matches!(
        world.get::<SettleTiltEase>(entity).unwrap().0,
        EaseFunction::CubicOut
    ));
}

// ── Behavior 34: build() produces correct bump components ──

#[test]
fn build_produces_correct_bump_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    assert!((world.get::<BumpPerfectWindow>(entity).unwrap().0 - 0.15).abs() < f32::EPSILON);
    assert!((world.get::<BumpEarlyWindow>(entity).unwrap().0 - 0.15).abs() < f32::EPSILON);
    assert!((world.get::<BumpLateWindow>(entity).unwrap().0 - 0.15).abs() < f32::EPSILON);
    assert!((world.get::<BumpPerfectCooldown>(entity).unwrap().0 - 0.0).abs() < f32::EPSILON);
    assert!((world.get::<BumpWeakCooldown>(entity).unwrap().0 - 0.15).abs() < f32::EPSILON);
    let bf = world.get::<BumpFeedback>(entity).unwrap();
    assert!((bf.duration - 0.15).abs() < f32::EPSILON);
    assert!((bf.peak - 24.0).abs() < f32::EPSILON);
    assert!((bf.peak_fraction - 0.3).abs() < f32::EPSILON);
    assert!(matches!(bf.rise_ease, EaseFunction::CubicOut));
    assert!(matches!(bf.fall_ease, EaseFunction::QuadraticIn));
}

// ── Behavior 35: build() produces correct spread component (degrees-to-radians) ──

#[test]
fn build_produces_correct_spread_component() {
    let def = test_breaker_definition(); // reflection_spread: 75.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - 75.0_f32.to_radians()).abs() < 1e-5);
}

#[test]
fn build_spread_zero_produces_zero() {
    let mut def = test_breaker_definition();
    def.reflection_spread = 0.0;
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - 0.0).abs() < f32::EPSILON);
}

#[test]
fn build_spread_180_produces_pi() {
    let mut def = test_breaker_definition();
    def.reflection_spread = 180.0;
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - std::f32::consts::PI).abs() < 1e-5);
}
