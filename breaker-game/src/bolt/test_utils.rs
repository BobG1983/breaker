//! Bolt domain test utilities.
//!
//! Consolidates shared test helpers used across 2+ bolt test suites.
//! Suite-specific helpers remain in their local `tests/helpers.rs`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::Bolt, definition::BoltDefinition},
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig, SizeBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
    },
};

/// Standard bolt definition matching values previously provided by
/// `BoltConfig::default()`, so existing position calculations remain valid.
///
/// Used by bolt-cell collision, bolt-breaker collision, and bolt-wall
/// collision test suites.
pub(crate) fn default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name:                 "Bolt".to_string(),
        base_speed:           400.0,
        min_speed:            200.0,
        max_speed:            800.0,
        radius:               8.0,
        base_damage:          10.0,
        effects:              vec![],
        color_rgb:            [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical:   5.0,
        min_radius:           None,
        max_radius:           None,
    }
}

/// Builds an `EffectStack<SpeedBoostConfig>` from a slice of f32 multipliers.
pub(crate) fn speed_stack(values: &[f32]) -> EffectStack<SpeedBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

/// Builds an `EffectStack<DamageBoostConfig>` from a slice of f32 multipliers.
pub(crate) fn damage_stack(values: &[f32]) -> EffectStack<DamageBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

/// Builds an `EffectStack<SizeBoostConfig>` from a slice of f32 multipliers.
pub(crate) fn size_stack(values: &[f32]) -> EffectStack<SizeBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

/// Builds an `EffectStack<PiercingConfig>` from a slice of u32 charge values.
pub(crate) fn piercing_stack(values: &[u32]) -> EffectStack<PiercingConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push("test".into(), PiercingConfig { charges: v });
    }
    stack
}

/// Spawns a bolt at the given position with the given velocity using the
/// builder with [`default_bolt_definition`].
pub(crate) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let def = default_bolt_definition();
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(Vec2::new(x, y))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}
