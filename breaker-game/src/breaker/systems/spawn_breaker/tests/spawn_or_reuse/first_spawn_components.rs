//! Tests for the `spawn_or_reuse_breaker` system — component detail checks.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::helpers::*;
use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerDeceleration, BreakerTilt,
            BumpEarlyWindow, BumpFeedback, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpState, BumpWeakCooldown, DashDuration, DashSpeedMultiplier, DashState,
            DashStateTimer, DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
        },
        definition::BreakerDefinition,
        messages::BreakerSpawned,
        registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    effect::effects::life_lost::LivesCount,
    shared::{
        BOLT_LAYER, BREAKER_LAYER, BaseHeight, BaseWidth,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

#[test]
fn spawned_breaker_has_rendered_components() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "spawned breaker should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "spawned breaker should have MeshMaterial2d"
    );
}

#[test]
fn only_selected_breaker_definition_is_used() {
    // Edge case: registry has multiple breakers but only SelectedBreaker is spawned
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<crate::shared::PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();

    let mut registry = BreakerRegistry::default();
    // Aegis with life_pool=3
    let aegis_def = BreakerDefinition {
        name: "Aegis".to_string(),
        life_pool: Some(3),
        ..BreakerDefinition::default()
    };
    // Chrono with life_pool=None
    let chrono_def = BreakerDefinition {
        name: "Chrono".to_string(),
        life_pool: None,
        ..BreakerDefinition::default()
    };
    registry.insert("Aegis".to_string(), aegis_def);
    registry.insert("Chrono".to_string(), chrono_def);
    app.insert_resource(registry);
    app.insert_resource(SelectedBreaker("Aegis".to_owned()));

    app.add_systems(Startup, super::super::super::system::spawn_or_reuse_breaker);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("breaker should have LivesCount");
    assert_eq!(
        lives.0,
        Some(3),
        "should use Aegis (life_pool=3), not Chrono (life_pool=None)"
    );
}

// ── Behavior 2: Movement components from definition ────────────────────

#[test]
fn spawned_breaker_has_movement_components() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let world = app.world();

    let accel = world
        .get::<BreakerAcceleration>(entity)
        .expect("should have BreakerAcceleration");
    assert!(
        (accel.0 - 3000.0).abs() < f32::EPSILON,
        "BreakerAcceleration should be 3000.0, got {}",
        accel.0
    );

    let decel = world
        .get::<BreakerDeceleration>(entity)
        .expect("should have BreakerDeceleration");
    assert!(
        (decel.0 - 2500.0).abs() < f32::EPSILON,
        "BreakerDeceleration should be 2500.0, got {}",
        decel.0
    );

    let easing = world
        .get::<DecelEasing>(entity)
        .expect("should have DecelEasing");
    assert_eq!(
        easing.ease,
        bevy::math::curve::easing::EaseFunction::QuadraticIn,
    );
    assert!(
        (easing.strength - 1.0).abs() < f32::EPSILON,
        "DecelEasing.strength should be 1.0, got {}",
        easing.strength
    );

    let dash_mult = world
        .get::<DashSpeedMultiplier>(entity)
        .expect("should have DashSpeedMultiplier");
    assert!(
        (dash_mult.0 - 4.0).abs() < f32::EPSILON,
        "DashSpeedMultiplier should be 4.0, got {}",
        dash_mult.0
    );

    let dash_dur = world
        .get::<DashDuration>(entity)
        .expect("should have DashDuration");
    assert!(
        (dash_dur.0 - 0.15).abs() < f32::EPSILON,
        "DashDuration should be 0.15, got {}",
        dash_dur.0
    );
}

// ── Behavior 3: Dash/tilt components from definition ───────────────────

#[test]
fn spawned_breaker_has_dash_tilt_components() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let world = app.world();

    let dash_tilt = world.get::<DashTilt>(entity).expect("should have DashTilt");
    assert!(
        (dash_tilt.0 - 15.0_f32.to_radians()).abs() < 1e-5,
        "DashTilt should be 15 degrees in radians, got {}",
        dash_tilt.0
    );

    let dash_tilt_ease = world
        .get::<DashTiltEase>(entity)
        .expect("should have DashTiltEase");
    assert_eq!(
        dash_tilt_ease.0,
        bevy::math::curve::easing::EaseFunction::QuadraticInOut,
    );

    let brake_tilt = world
        .get::<BrakeTilt>(entity)
        .expect("should have BrakeTilt");
    assert!(
        (brake_tilt.angle - 25.0_f32.to_radians()).abs() < 1e-5,
        "BrakeTilt.angle should be 25 degrees in radians, got {}",
        brake_tilt.angle
    );
    assert!(
        (brake_tilt.duration - 0.2).abs() < f32::EPSILON,
        "BrakeTilt.duration should be 0.2, got {}",
        brake_tilt.duration
    );
    assert_eq!(
        brake_tilt.ease,
        bevy::math::curve::easing::EaseFunction::CubicInOut,
    );

    let brake_decel = world
        .get::<BrakeDecel>(entity)
        .expect("should have BrakeDecel");
    assert!(
        (brake_decel.0 - 2.0).abs() < f32::EPSILON,
        "BrakeDecel should be 2.0, got {}",
        brake_decel.0
    );

    let settle_dur = world
        .get::<SettleDuration>(entity)
        .expect("should have SettleDuration");
    assert!(
        (settle_dur.0 - 0.25).abs() < f32::EPSILON,
        "SettleDuration should be 0.25, got {}",
        settle_dur.0
    );

    let settle_ease = world
        .get::<SettleTiltEase>(entity)
        .expect("should have SettleTiltEase");
    assert_eq!(
        settle_ease.0,
        bevy::math::curve::easing::EaseFunction::CubicOut,
    );
}

// ── Behavior 4: Bump components from definition ────────────────────────

#[test]
fn spawned_breaker_has_bump_components() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let world = app.world();

    let pw = world
        .get::<BumpPerfectWindow>(entity)
        .expect("BumpPerfectWindow");
    assert!(
        (pw.0 - 0.15).abs() < f32::EPSILON,
        "BumpPerfectWindow should be 0.15, got {}",
        pw.0
    );

    let ew = world
        .get::<BumpEarlyWindow>(entity)
        .expect("BumpEarlyWindow");
    assert!(
        (ew.0 - 0.15).abs() < f32::EPSILON,
        "BumpEarlyWindow should be 0.15, got {}",
        ew.0
    );

    let lw = world.get::<BumpLateWindow>(entity).expect("BumpLateWindow");
    assert!(
        (lw.0 - 0.15).abs() < f32::EPSILON,
        "BumpLateWindow should be 0.15, got {}",
        lw.0
    );

    let pc = world
        .get::<BumpPerfectCooldown>(entity)
        .expect("BumpPerfectCooldown");
    assert!(
        (pc.0 - 0.0).abs() < f32::EPSILON,
        "BumpPerfectCooldown should be 0.0, got {}",
        pc.0
    );

    let wc = world
        .get::<BumpWeakCooldown>(entity)
        .expect("BumpWeakCooldown");
    assert!(
        (wc.0 - 0.15).abs() < f32::EPSILON,
        "BumpWeakCooldown should be 0.15, got {}",
        wc.0
    );

    let feedback = world.get::<BumpFeedback>(entity).expect("BumpFeedback");
    assert!((feedback.duration - 0.15).abs() < f32::EPSILON);
    assert!((feedback.peak - 24.0).abs() < f32::EPSILON);
    assert!((feedback.peak_fraction - 0.3).abs() < f32::EPSILON);
    assert_eq!(
        feedback.rise_ease,
        bevy::math::curve::easing::EaseFunction::CubicOut
    );
    assert_eq!(
        feedback.fall_ease,
        bevy::math::curve::easing::EaseFunction::QuadraticIn
    );
}

// ── Behavior 5: Size constraint components ─────────────────────────────

#[test]
fn spawned_breaker_has_size_constraint_components() {
    // Given: width=120.0, height=20.0, min_w/max_w/min_h/max_h all None (default)
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let world = app.world();

    let bw = world.get::<BaseWidth>(entity).expect("BaseWidth");
    assert!(
        (bw.0 - 120.0).abs() < f32::EPSILON,
        "BaseWidth should be 120.0, got {}",
        bw.0
    );

    let bh = world.get::<BaseHeight>(entity).expect("BaseHeight");
    assert!(
        (bh.0 - 20.0).abs() < f32::EPSILON,
        "BaseHeight should be 20.0, got {}",
        bh.0
    );

    let min_w = world.get::<MinWidth>(entity).expect("MinWidth");
    assert!(
        (min_w.0 - 60.0).abs() < f32::EPSILON,
        "MinWidth should be 60.0 (0.5 * 120), got {}",
        min_w.0
    );

    let max_w = world.get::<MaxWidth>(entity).expect("MaxWidth");
    assert!(
        (max_w.0 - 600.0).abs() < f32::EPSILON,
        "MaxWidth should be 600.0 (5.0 * 120), got {}",
        max_w.0
    );

    let min_h = world.get::<MinHeight>(entity).expect("MinHeight");
    assert!(
        (min_h.0 - 10.0).abs() < f32::EPSILON,
        "MinHeight should be 10.0 (0.5 * 20), got {}",
        min_h.0
    );

    let max_h = world.get::<MaxHeight>(entity).expect("MaxHeight");
    assert!(
        (max_h.0 - 100.0).abs() < f32::EPSILON,
        "MaxHeight should be 100.0 (5.0 * 20), got {}",
        max_h.0
    );
}

// ── Behavior 6: Physics components ─────────────────────────────────────

#[test]
fn spawned_breaker_has_aabb2d_matching_dimensions() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let aabb = app
        .world()
        .get::<Aabb2D>(entity)
        .expect("breaker should have Aabb2D");
    assert_eq!(aabb.center, Vec2::ZERO, "Aabb2D center should be ZERO");
    assert!(
        (aabb.half_extents.x - 60.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 10.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (60.0, 10.0), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y,
    );
}

#[test]
fn spawned_breaker_has_collision_layers() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("breaker should have CollisionLayers");
    assert_eq!(
        layers.membership, BREAKER_LAYER,
        "breaker membership should be BREAKER_LAYER (0x{BREAKER_LAYER:02X}), got 0x{:02X}",
        layers.membership,
    );
    assert_eq!(
        layers.mask, BOLT_LAYER,
        "breaker mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers.mask,
    );
}

// ── Behavior 7: Default dynamic state ──────────────────────────────────

#[test]
fn spawned_breaker_has_default_dynamic_state() {
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let world = app.world();

    let vel = world.get::<Velocity2D>(entity).expect("Velocity2D");
    assert_eq!(vel.0, Vec2::ZERO, "Velocity2D should be ZERO");

    let dash = world.get::<DashState>(entity).expect("DashState");
    assert_eq!(*dash, DashState::Idle, "DashState should be Idle");

    let tilt = world.get::<BreakerTilt>(entity).expect("BreakerTilt");
    assert!(
        (tilt.angle).abs() < f32::EPSILON,
        "tilt.angle should be 0.0"
    );
    assert!(
        (tilt.ease_start).abs() < f32::EPSILON,
        "tilt.ease_start should be 0.0"
    );
    assert!(
        (tilt.ease_target).abs() < f32::EPSILON,
        "tilt.ease_target should be 0.0"
    );

    let bump = world.get::<BumpState>(entity).expect("BumpState");
    assert!(!bump.active, "bump should be inactive");
    assert!(
        (bump.timer).abs() < f32::EPSILON,
        "bump.timer should be 0.0"
    );
    assert!(
        (bump.post_hit_timer).abs() < f32::EPSILON,
        "bump.post_hit_timer should be 0.0"
    );
    assert!(
        (bump.cooldown).abs() < f32::EPSILON,
        "bump.cooldown should be 0.0"
    );

    let timer = world.get::<DashStateTimer>(entity).expect("DashStateTimer");
    assert!(
        (timer.remaining).abs() < f32::EPSILON,
        "timer.remaining should be 0.0"
    );
}
