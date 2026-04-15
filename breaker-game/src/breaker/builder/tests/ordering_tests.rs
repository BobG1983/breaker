use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::{super::core::*, helpers::test_breaker_definition};
use crate::{
    breaker::{components::BreakerBaseY, definition::BreakerDefinition},
    prelude::*,
    shared::BaseWidth,
};

fn default_movement() -> MovementSettings {
    let defaults = BreakerDefinition::default();
    MovementSettings {
        max_speed:           defaults.max_speed,
        acceleration:        defaults.acceleration,
        deceleration:        defaults.deceleration,
        decel_ease:          defaults.decel_ease,
        decel_ease_strength: defaults.decel_ease_strength,
    }
}

fn default_dashing() -> DashSettings {
    let defaults = BreakerDefinition::default();
    DashSettings {
        dash:   DashParams {
            speed_multiplier: defaults.dash_speed_multiplier,
            duration:         defaults.dash_duration,
            tilt_angle:       defaults.dash_tilt_angle,
            tilt_ease:        defaults.dash_tilt_ease,
        },
        brake:  BrakeParams {
            tilt_angle:       defaults.brake_tilt_angle,
            tilt_duration:    defaults.brake_tilt_duration,
            tilt_ease:        defaults.brake_tilt_ease,
            decel_multiplier: defaults.brake_decel_multiplier,
        },
        settle: SettleParams {
            duration:  defaults.settle_duration,
            tilt_ease: defaults.settle_tilt_ease,
        },
    }
}

fn default_bump() -> BumpSettings {
    let defaults = BreakerDefinition::default();
    BumpSettings {
        perfect_window:   defaults.perfect_window,
        early_window:     defaults.early_window,
        late_window:      defaults.late_window,
        perfect_cooldown: defaults.perfect_bump_cooldown,
        weak_cooldown:    defaults.weak_bump_cooldown,
        feedback:         BumpFeedbackSettings {
            duration:      defaults.bump_visual_duration,
            peak:          defaults.bump_visual_peak,
            peak_fraction: defaults.bump_visual_peak_fraction,
            rise_ease:     defaults.bump_visual_rise_ease,
            fall_ease:     defaults.bump_visual_fall_ease,
        },
    }
}

// ── Behavior 57: Typestate transitions can be called in any order ──

#[test]
fn different_ordering_produces_identical_entities() {
    let defaults = BreakerDefinition::default();
    let mut world = World::new();

    // Order A: dimensions first, then movement, dashing, spread, bump, headless, primary
    let entity_a = Breaker::builder()
        .dimensions(defaults.width, defaults.height, defaults.y_position)
        .movement(default_movement())
        .dashing(default_dashing())
        .spread(defaults.reflection_spread)
        .bump(default_bump())
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    // Order B: primary first, then headless, bump, spread, dashing, movement, dimensions
    let entity_b = Breaker::builder()
        .primary()
        .headless()
        .bump(default_bump())
        .spread(defaults.reflection_spread)
        .dashing(default_dashing())
        .movement(default_movement())
        .dimensions(defaults.width, defaults.height, defaults.y_position)
        .spawn(&mut world.commands());
    world.flush();

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
    let entity = Breaker::builder()
        .definition(&def)
        .with_max_speed(700.0)
        .with_width(200.0)
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

    let bw = world.get::<BaseWidth>(entity);
    assert!(bw.is_some(), "entity should have BaseWidth");
    assert!(
        (bw.unwrap().0 - 200.0).abs() < f32::EPSILON,
        "BaseWidth should be overridden to 200.0"
    );
}
