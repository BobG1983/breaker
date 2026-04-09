use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};
use rantzsoft_stateflow::CleanupOnExit;

use super::helpers::test_breaker_definition;
use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
            BreakerInitialized, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow,
            BumpFeedback, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState,
            BumpWeakCooldown, DashDuration, DashSpeedMultiplier, DashState, DashStateTimer,
            DashTilt, DashTiltEase, DecelEasing, PrimaryBreaker, SettleDuration, SettleTiltEase,
        },
        definition::BreakerDefinition,
    },
    effect::effects::life_lost::LivesCount,
    shared::{
        BaseHeight, BaseWidth, GameDrawLayer,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
    state::types::RunState,
};

// ── Behavior 28: build() on a headless primary breaker produces all core components ──

#[test]
fn build_headless_primary_has_breaker_marker() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "should have CleanupOnExit<RunState>"
    );
}

#[test]
fn build_headless_primary_has_default_state_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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

    // Headless builds do NOT include GameDrawLayer (only Rendered builds do)
    let layer = world.get::<GameDrawLayer>(entity);
    assert!(
        layer.is_none(),
        "headless build should NOT have GameDrawLayer"
    );

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "should have LivesCount");
}

// ── Behavior 29: build() produces correct spatial components ──

#[test]
fn build_produces_correct_spatial_components() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0, y_position: -250.0
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity);
    assert!(pos.is_some(), "should have Position2D");
    let pos = pos.unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "Position2D.x should be 0.0"
    );
    assert!(
        (pos.0.y - defaults.y_position).abs() < f32::EPSILON,
        "Position2D.y should be default y_position"
    );

    let prev = world.get::<PreviousPosition>(entity);
    assert!(prev.is_some(), "should have PreviousPosition");
    let prev = prev.unwrap();
    assert!(
        (prev.0.x - 0.0).abs() < f32::EPSILON,
        "PreviousPosition.x should be 0.0"
    );
    assert!(
        (prev.0.y - defaults.y_position).abs() < f32::EPSILON,
        "PreviousPosition.y should be default y_position"
    );

    let scale = world.get::<Scale2D>(entity);
    assert!(scale.is_some(), "should have Scale2D");
    assert!(
        (scale.unwrap().x - defaults.width).abs() < f32::EPSILON,
        "Scale2D.x should be default width"
    );
    assert!(
        (scale.unwrap().y - defaults.height).abs() < f32::EPSILON,
        "Scale2D.y should be default height"
    );

    let prev_scale = world.get::<PreviousScale>(entity);
    assert!(prev_scale.is_some(), "should have PreviousScale");
    assert!((prev_scale.unwrap().x - defaults.width).abs() < f32::EPSILON);
    assert!((prev_scale.unwrap().y - defaults.height).abs() < f32::EPSILON);

    let aabb = world.get::<Aabb2D>(entity);
    assert!(aabb.is_some(), "should have Aabb2D");
    assert!((aabb.unwrap().half_extents.x - defaults.width / 2.0).abs() < f32::EPSILON);
    assert!((aabb.unwrap().half_extents.y - defaults.height / 2.0).abs() < f32::EPSILON);
}

// ── Behavior 30: build() produces correct physics components ──

#[test]
fn build_produces_correct_collision_layers() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    assert!((world.get::<BaseWidth>(entity).unwrap().0 - defaults.width).abs() < f32::EPSILON);
    assert!((world.get::<BaseHeight>(entity).unwrap().0 - defaults.height).abs() < f32::EPSILON);
    assert!(
        defaults
            .width
            .mul_add(-0.5, world.get::<MinWidth>(entity).unwrap().0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        defaults
            .width
            .mul_add(-5.0, world.get::<MaxWidth>(entity).unwrap().0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        defaults
            .height
            .mul_add(-0.5, world.get::<MinHeight>(entity).unwrap().0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        defaults
            .height
            .mul_add(-5.0, world.get::<MaxHeight>(entity).unwrap().0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BreakerBaseY>(entity).unwrap().0 - defaults.y_position).abs() < f32::EPSILON
    );
}

// ── Behavior 32: build() produces correct movement components ──

#[test]
fn build_produces_correct_movement_components() {
    let def = test_breaker_definition();
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    assert!((world.get::<MaxSpeed>(entity).unwrap().0 - defaults.max_speed).abs() < f32::EPSILON);
    assert!(
        (world.get::<BreakerAcceleration>(entity).unwrap().0 - defaults.acceleration).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BreakerDeceleration>(entity).unwrap().0 - defaults.deceleration).abs()
            < f32::EPSILON
    );
    let de = world.get::<DecelEasing>(entity).unwrap();
    assert_eq!(de.ease, defaults.decel_ease);
    assert!((de.strength - defaults.decel_ease_strength).abs() < f32::EPSILON);
}

// ── Behavior 33: build() produces correct dash components ──

#[test]
fn build_produces_correct_dash_components() {
    let def = test_breaker_definition();
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        (world.get::<DashSpeedMultiplier>(entity).unwrap().0 - defaults.dash_speed_multiplier)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<DashDuration>(entity).unwrap().0 - defaults.dash_duration).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<DashTilt>(entity).unwrap().0 - defaults.dash_tilt_angle.to_radians()).abs()
            < 1e-5
    );
    assert_eq!(
        world.get::<DashTiltEase>(entity).unwrap().0,
        defaults.dash_tilt_ease
    );
    let bt = world.get::<BrakeTilt>(entity).unwrap();
    assert!((bt.angle - defaults.brake_tilt_angle.to_radians()).abs() < 1e-5);
    assert!((bt.duration - defaults.brake_tilt_duration).abs() < f32::EPSILON);
    assert_eq!(bt.ease, defaults.brake_tilt_ease);
    assert!(
        (world.get::<BrakeDecel>(entity).unwrap().0 - defaults.brake_decel_multiplier).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<SettleDuration>(entity).unwrap().0 - defaults.settle_duration).abs()
            < f32::EPSILON
    );
    assert_eq!(
        world.get::<SettleTiltEase>(entity).unwrap().0,
        defaults.settle_tilt_ease
    );
}

// ── Behavior 34: build() produces correct bump components ──

#[test]
fn build_produces_correct_bump_components() {
    let def = test_breaker_definition();
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        (world.get::<BumpPerfectWindow>(entity).unwrap().0 - defaults.perfect_window).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BumpEarlyWindow>(entity).unwrap().0 - defaults.early_window).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BumpLateWindow>(entity).unwrap().0 - defaults.late_window).abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BumpPerfectCooldown>(entity).unwrap().0 - defaults.perfect_bump_cooldown)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (world.get::<BumpWeakCooldown>(entity).unwrap().0 - defaults.weak_bump_cooldown).abs()
            < f32::EPSILON
    );
    let bf = world.get::<BumpFeedback>(entity).unwrap();
    assert!((bf.duration - defaults.bump_visual_duration).abs() < f32::EPSILON);
    assert!((bf.peak - defaults.bump_visual_peak).abs() < f32::EPSILON);
    assert!((bf.peak_fraction - defaults.bump_visual_peak_fraction).abs() < f32::EPSILON);
    assert_eq!(bf.rise_ease, defaults.bump_visual_rise_ease);
    assert_eq!(bf.fall_ease, defaults.bump_visual_fall_ease);
}

// ── Behavior 35: build() produces correct spread component (degrees-to-radians) ──

#[test]
fn build_produces_correct_spread_component() {
    let def = test_breaker_definition(); // reflection_spread: 75.0
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - defaults.reflection_spread.to_radians()).abs() < 1e-5);
}

#[test]
fn build_spread_zero_produces_zero() {
    let mut def = test_breaker_definition();
    def.reflection_spread = 0.0;
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - 0.0).abs() < f32::EPSILON);
}

#[test]
fn build_spread_180_produces_pi() {
    let mut def = test_breaker_definition();
    def.reflection_spread = 180.0;
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(spread.is_some(), "should have BreakerReflectionSpread");
    assert!((spread.unwrap().0 - std::f32::consts::PI).abs() < 1e-5);
}
