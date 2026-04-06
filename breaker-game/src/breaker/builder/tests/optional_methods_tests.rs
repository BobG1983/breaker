use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, PreviousScale, Scale2D};

use super::helpers::test_breaker_definition;
use crate::{
    breaker::components::{Breaker, BreakerBaseY, BreakerReflectionSpread},
    effect::effects::life_lost::LivesCount,
    shared::{BaseHeight, BaseWidth},
};

// ── Behavior 19: .with_max_speed() overrides definition max_speed ──

#[test]
fn with_max_speed_overrides_definition_value() {
    let def = test_breaker_definition(); // max_speed: 1000.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_max_speed(700.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let ms = world.get::<MaxSpeed>(entity);
    assert!(ms.is_some(), "entity should have MaxSpeed");
    assert!(
        (ms.unwrap().0 - 700.0).abs() < f32::EPSILON,
        "MaxSpeed should be overridden to 700.0"
    );
}

#[test]
fn with_max_speed_zero_stores_zero() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_max_speed(0.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let ms = world.get::<MaxSpeed>(entity);
    assert!(ms.is_some(), "entity should have MaxSpeed");
    assert!(
        (ms.unwrap().0 - 0.0).abs() < f32::EPSILON,
        "MaxSpeed should be 0.0"
    );
}

// ── Behavior 20: .with_width() overrides definition width ──

#[test]
fn with_width_overrides_definition_value() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_width(200.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let bw = world.get::<BaseWidth>(entity);
    assert!(bw.is_some(), "entity should have BaseWidth");
    assert!(
        (bw.unwrap().0 - 200.0).abs() < f32::EPSILON,
        "BaseWidth should be 200.0"
    );

    let scale = world.get::<Scale2D>(entity);
    assert!(scale.is_some(), "entity should have Scale2D");
    let scale = scale.unwrap();
    assert!(
        (scale.x - 200.0).abs() < f32::EPSILON,
        "Scale2D.x should be 200.0"
    );
    assert!(
        (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D.y should remain 20.0"
    );

    let aabb = world.get::<Aabb2D>(entity);
    assert!(aabb.is_some(), "entity should have Aabb2D");
    assert!(
        (aabb.unwrap().half_extents.x - 100.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.x should be 100.0"
    );

    let ps = world.get::<PreviousScale>(entity);
    assert!(ps.is_some(), "entity should have PreviousScale");
    assert!(
        (ps.unwrap().x - 200.0).abs() < f32::EPSILON,
        "PreviousScale.x should be 200.0"
    );
}

// ── Behavior 21: .with_height() overrides definition height ──

#[test]
fn with_height_overrides_definition_value() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_height(30.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let bh = world.get::<BaseHeight>(entity);
    assert!(bh.is_some(), "entity should have BaseHeight");
    assert!(
        (bh.unwrap().0 - 30.0).abs() < f32::EPSILON,
        "BaseHeight should be 30.0"
    );

    let scale = world.get::<Scale2D>(entity);
    assert!(scale.is_some(), "entity should have Scale2D");
    let scale = scale.unwrap();
    assert!(
        (scale.x - 120.0).abs() < f32::EPSILON,
        "Scale2D.x should remain 120.0"
    );
    assert!(
        (scale.y - 30.0).abs() < f32::EPSILON,
        "Scale2D.y should be 30.0"
    );

    let aabb = world.get::<Aabb2D>(entity);
    assert!(aabb.is_some(), "entity should have Aabb2D");
    assert!(
        (aabb.unwrap().half_extents.y - 15.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.y should be 15.0"
    );

    let ps = world.get::<PreviousScale>(entity);
    assert!(ps.is_some(), "entity should have PreviousScale");
    let ps = ps.unwrap();
    assert!(
        (ps.x - 120.0).abs() < f32::EPSILON,
        "PreviousScale.x should remain 120.0"
    );
    assert!(
        (ps.y - 30.0).abs() < f32::EPSILON,
        "PreviousScale.y should be 30.0"
    );
}

// ── Behavior 22: .with_y_position() overrides definition y_position ──

#[test]
fn with_y_position_overrides_definition_value() {
    let def = test_breaker_definition(); // y_position: -250.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_y_position(-300.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let by = world.get::<BreakerBaseY>(entity);
    assert!(by.is_some(), "entity should have BreakerBaseY");
    assert!(
        (by.unwrap().0 - (-300.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be -300.0"
    );

    let pos = world.get::<Position2D>(entity);
    assert!(pos.is_some(), "entity should have Position2D");
    assert!(
        (pos.unwrap().0.y - (-300.0)).abs() < f32::EPSILON,
        "Position2D.y should be -300.0"
    );
}

// ── Behavior 23: .with_reflection_spread() overrides definition spread ──

#[test]
fn with_reflection_spread_overrides_definition_value() {
    let def = test_breaker_definition(); // reflection_spread: 75.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_reflection_spread(60.0)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(
        spread.is_some(),
        "entity should have BreakerReflectionSpread"
    );
    assert!((spread.unwrap().0 - 60.0_f32.to_radians()).abs() < 1e-5);
}

// ── Behavior 24: .with_lives() sets LivesCount ──

#[test]
fn with_lives_some_sets_lives_count() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_lives(Some(3))
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(lives.unwrap().0, Some(3), "LivesCount should be Some(3)");
}

#[test]
fn with_lives_none_sets_infinite_lives() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_lives(None)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(
        lives.unwrap().0,
        None,
        "LivesCount should be None (infinite)"
    );
}

#[test]
fn with_lives_zero_stores_zero() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_lives(Some(0))
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(lives.unwrap().0, Some(0), "LivesCount should be Some(0)");
}

// ── Behavior 25: Without .with_lives(), LivesCount uses definition's life_pool ──

#[test]
fn without_with_lives_uses_definition_life_pool() {
    let mut def = test_breaker_definition();
    def.life_pool = Some(3);
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(
        lives.unwrap().0,
        Some(3),
        "LivesCount should be Some(3) from definition"
    );
}

#[test]
fn without_with_lives_definition_none_produces_infinite() {
    let def = test_breaker_definition(); // life_pool: None
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(
        lives.unwrap().0,
        None,
        "LivesCount should be None from definition"
    );
}

#[test]
fn without_definition_without_with_lives_defaults_to_infinite() {
    let mut world = World::new();
    let bundle = Breaker::builder()
        .dimensions(120.0, 20.0, -250.0)
        .movement(super::super::core::MovementSettings {
            max_speed: 1000.0,
            acceleration: 6000.0,
            deceleration: 5000.0,
            decel_ease: EaseFunction::QuadraticIn,
            decel_ease_strength: 1.0,
        })
        .dashing(super::super::core::DashSettings {
            dash: super::super::core::DashParams {
                speed_multiplier: 4.0,
                duration: 0.15,
                tilt_angle: 15.0,
                tilt_ease: EaseFunction::QuadraticInOut,
            },
            brake: super::super::core::BrakeParams {
                tilt_angle: 25.0,
                tilt_duration: 0.2,
                tilt_ease: EaseFunction::CubicInOut,
                decel_multiplier: 2.0,
            },
            settle: super::super::core::SettleParams {
                duration: 0.25,
                tilt_ease: EaseFunction::CubicOut,
            },
        })
        .spread(75.0)
        .bump(super::super::core::BumpSettings {
            perfect_window: 0.15,
            early_window: 0.15,
            late_window: 0.15,
            perfect_cooldown: 0.0,
            weak_cooldown: 0.15,
            feedback: super::super::core::BumpFeedbackSettings {
                duration: 0.15,
                peak: 24.0,
                peak_fraction: 0.3,
                rise_ease: EaseFunction::CubicOut,
                fall_ease: EaseFunction::QuadraticIn,
            },
        })
        .headless()
        .primary()
        .build();
    let entity = world.spawn(bundle).id();

    let lives = world.get::<LivesCount>(entity);
    assert!(
        lives.is_some(),
        "entity should have LivesCount even without definition"
    );
    assert_eq!(
        lives.unwrap().0,
        None,
        "LivesCount should default to None (infinite) without definition or with_lives"
    );
}

// ── Behavior 26: .with_effects() stores effects for spawn ──
// (Tested more thoroughly in spawn_tests.rs, but verify it compiles here)

#[test]
fn with_effects_compiles_and_stores() {
    let def = test_breaker_definition();
    let _builder = Breaker::builder()
        .definition(&def)
        .with_effects(vec![])
        .headless()
        .primary();
    // Compilation is the assertion.
}

// ── Behavior 27: .with_effects() overrides definition effects ──

#[test]
fn with_effects_overrides_definition_effects() {
    // This is tested more thoroughly in spawn_tests.rs where we verify
    // the dispatched effects. Here we just verify the chain compiles.
    let mut def = test_breaker_definition();
    def.effects = vec![crate::effect::RootEffect::On {
        target: crate::effect::Target::Breaker,
        then: vec![crate::effect::EffectNode::When {
            trigger: crate::effect::Trigger::BoltLost,
            then: vec![crate::effect::EffectNode::Do(
                crate::effect::EffectKind::LoseLife,
            )],
        }],
    }];
    let _builder = Breaker::builder()
        .definition(&def)
        .with_effects(vec![]) // explicitly empty overrides definition
        .headless()
        .primary();
}
