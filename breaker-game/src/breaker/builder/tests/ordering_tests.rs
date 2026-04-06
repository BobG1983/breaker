use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_spatial2d::components::MaxSpeed;

use super::{super::core::*, helpers::test_breaker_definition};
use crate::{
    breaker::components::{Breaker, BreakerBaseY},
    shared::BaseWidth,
};

fn default_movement() -> MovementSettings {
    MovementSettings {
        max_speed: 1000.0,
        acceleration: 6000.0,
        deceleration: 5000.0,
        decel_ease: EaseFunction::QuadraticIn,
        decel_ease_strength: 1.0,
    }
}

fn default_dashing() -> DashSettings {
    DashSettings {
        dash: DashParams {
            speed_multiplier: 4.0,
            duration: 0.15,
            tilt_angle: 15.0,
            tilt_ease: EaseFunction::QuadraticInOut,
        },
        brake: BrakeParams {
            tilt_angle: 25.0,
            tilt_duration: 0.2,
            tilt_ease: EaseFunction::CubicInOut,
            decel_multiplier: 2.0,
        },
        settle: SettleParams {
            duration: 0.25,
            tilt_ease: EaseFunction::CubicOut,
        },
    }
}

fn default_bump() -> BumpSettings {
    BumpSettings {
        perfect_window: 0.15,
        early_window: 0.15,
        late_window: 0.15,
        perfect_cooldown: 0.0,
        weak_cooldown: 0.15,
        feedback: BumpFeedbackSettings {
            duration: 0.15,
            peak: 24.0,
            peak_fraction: 0.3,
            rise_ease: EaseFunction::CubicOut,
            fall_ease: EaseFunction::QuadraticIn,
        },
    }
}

// ── Behavior 57: Typestate transitions can be called in any order ──

#[test]
fn different_ordering_produces_identical_entities() {
    let mut world = World::new();

    // Order A: dimensions first, then movement, dashing, spread, bump, headless, primary
    let bundle_a = Breaker::builder()
        .dimensions(120.0, 20.0, -250.0)
        .movement(default_movement())
        .dashing(default_dashing())
        .spread(75.0)
        .bump(default_bump())
        .headless()
        .primary()
        .build();
    let entity_a = world.spawn(bundle_a).id();

    // Order B: primary first, then headless, bump, spread, dashing, movement, dimensions
    let bundle_b = Breaker::builder()
        .primary()
        .headless()
        .bump(default_bump())
        .spread(75.0)
        .dashing(default_dashing())
        .movement(default_movement())
        .dimensions(120.0, 20.0, -250.0)
        .build();
    let entity_b = world.spawn(bundle_b).id();

    // Both should have the same MaxSpeed
    let ms_a = world.get::<MaxSpeed>(entity_a);
    let ms_b = world.get::<MaxSpeed>(entity_b);
    assert!(ms_a.is_some(), "order A entity should have MaxSpeed");
    assert!(ms_b.is_some(), "order B entity should have MaxSpeed");
    assert!(
        (ms_a.unwrap().0 - ms_b.unwrap().0).abs() < f32::EPSILON,
        "MaxSpeed should be identical regardless of ordering"
    );

    // Both should have the same BaseWidth
    let bw_a = world.get::<BaseWidth>(entity_a);
    let bw_b = world.get::<BaseWidth>(entity_b);
    assert!(bw_a.is_some(), "order A entity should have BaseWidth");
    assert!(bw_b.is_some(), "order B entity should have BaseWidth");
    assert!(
        (bw_a.unwrap().0 - bw_b.unwrap().0).abs() < f32::EPSILON,
        "BaseWidth should be identical regardless of ordering"
    );

    // Both should have the same BreakerBaseY
    let base_y_a = world.get::<BreakerBaseY>(entity_a);
    let base_y_b = world.get::<BreakerBaseY>(entity_b);
    assert!(
        base_y_a.is_some(),
        "order A entity should have BreakerBaseY"
    );
    assert!(
        base_y_b.is_some(),
        "order B entity should have BreakerBaseY"
    );
    assert!(
        (base_y_a.unwrap().0 - base_y_b.unwrap().0).abs() < f32::EPSILON,
        "BreakerBaseY should be identical regardless of ordering"
    );
}

// ── Behavior 58: Optional .with_*() methods after .definition() before terminal ──

#[test]
fn with_overrides_after_definition_before_build() {
    let def = test_breaker_definition(); // max_speed: 1000.0, width: 120.0
    let mut world = World::new();
    let bundle = Breaker::builder()
        .definition(&def)
        .with_max_speed(700.0)
        .with_width(200.0)
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

    let bw = world.get::<BaseWidth>(entity);
    assert!(bw.is_some(), "entity should have BaseWidth");
    assert!(
        (bw.unwrap().0 - 200.0).abs() < f32::EPSILON,
        "BaseWidth should be overridden to 200.0"
    );
}
