use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D,
};

use super::helpers::test_breaker_definition;
use crate::{
    breaker::{
        components::{Breaker, BreakerBaseY, BreakerReflectionSpread},
        definition::BreakerDefinition,
    },
    effect::effects::life_lost::LivesCount,
    shared::{BaseHeight, BaseWidth},
};

// ── Behavior 19: .with_max_speed() overrides definition max_speed ──

#[test]
fn with_max_speed_overrides_definition_value() {
    let def = test_breaker_definition(); // max_speed: 1000.0
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .with_max_speed(700.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .with_max_speed(0.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let def = test_breaker_definition();
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .with_width(200.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
        (scale.y - defaults.height).abs() < f32::EPSILON,
        "Scale2D.y should remain default height"
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
    let def = test_breaker_definition();
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .with_height(30.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
        (scale.x - defaults.width).abs() < f32::EPSILON,
        "Scale2D.x should remain default width"
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
        (ps.x - defaults.width).abs() < f32::EPSILON,
        "PreviousScale.x should remain default width"
    );
    assert!(
        (ps.y - 30.0).abs() < f32::EPSILON,
        "PreviousScale.y should be 30.0"
    );
}

// ── at_position() sets both x and y ──

#[test]
fn at_position_sets_both_x_and_y() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .at_position(Vec2::new(150.0, -300.0))
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let pos = world
        .get::<Position2D>(entity)
        .expect("should have Position2D");
    assert!(
        (pos.0.x - 150.0).abs() < f32::EPSILON,
        "Position2D.x should be 150.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Position2D.y should be -300.0, got {}",
        pos.0.y
    );

    let prev = world
        .get::<PreviousPosition>(entity)
        .expect("should have PreviousPosition");
    assert!(
        (prev.0.x - 150.0).abs() < f32::EPSILON,
        "PreviousPosition.x should be 150.0"
    );

    let by = world
        .get::<BreakerBaseY>(entity)
        .expect("should have BreakerBaseY");
    assert!(
        (by.0 - (-300.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be -300.0"
    );
}

#[test]
fn at_position_zero_overrides_definition_defaults() {
    let def = test_breaker_definition(); // y_position: -250.0
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let pos = world.get::<Position2D>(entity).expect("should exist");
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "explicit zero x should not fall back to any default"
    );
    assert!(
        pos.0.y.abs() < f32::EPSILON,
        "explicit zero y should override definition y_position (-250.0)"
    );
}

// ── Behavior 22: .with_y_position() overrides definition y_position ──

#[test]
fn with_y_position_overrides_definition_value() {
    let def = test_breaker_definition(); // y_position: -250.0
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .with_y_position(-300.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .with_reflection_spread(60.0)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .with_lives(Some(3))
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let lives = world.get::<LivesCount>(entity);
    assert!(lives.is_some(), "entity should have LivesCount");
    assert_eq!(lives.unwrap().0, Some(3), "LivesCount should be Some(3)");
}

#[test]
fn with_lives_none_sets_infinite_lives() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .with_lives(None)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .with_lives(Some(0))
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
    let defaults = BreakerDefinition::default();
    let mut world = World::new();
    let entity = Breaker::builder()
        .dimensions(defaults.width, defaults.height, defaults.y_position)
        .movement(crate::breaker::builder::core::MovementSettings {
            max_speed: defaults.max_speed,

            acceleration: defaults.acceleration,

            deceleration: defaults.deceleration,

            decel_ease: defaults.decel_ease,

            decel_ease_strength: defaults.decel_ease_strength,
        })
        .dashing(crate::breaker::builder::core::DashSettings {
            dash: crate::breaker::builder::core::DashParams {
                speed_multiplier: defaults.dash_speed_multiplier,

                duration: defaults.dash_duration,

                tilt_angle: defaults.dash_tilt_angle,

                tilt_ease: defaults.dash_tilt_ease,
            },

            brake: crate::breaker::builder::core::BrakeParams {
                tilt_angle: defaults.brake_tilt_angle,

                tilt_duration: defaults.brake_tilt_duration,

                tilt_ease: defaults.brake_tilt_ease,

                decel_multiplier: defaults.brake_decel_multiplier,
            },

            settle: crate::breaker::builder::core::SettleParams {
                duration: defaults.settle_duration,

                tilt_ease: defaults.settle_tilt_ease,
            },
        })
        .spread(defaults.reflection_spread)
        .bump(crate::breaker::builder::core::BumpSettings {
            perfect_window: defaults.perfect_window,

            early_window: defaults.early_window,

            late_window: defaults.late_window,

            perfect_cooldown: defaults.perfect_bump_cooldown,

            weak_cooldown: defaults.weak_bump_cooldown,

            feedback: crate::breaker::builder::core::BumpFeedbackSettings {
                duration: defaults.bump_visual_duration,

                peak: defaults.bump_visual_peak,

                peak_fraction: defaults.bump_visual_peak_fraction,

                rise_ease: defaults.bump_visual_rise_ease,

                fall_ease: defaults.bump_visual_fall_ease,
            },
        })
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

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
