pub(super) use bevy::prelude::*;
pub(super) use rantzsoft_spatial2d::components::Position2D;

pub(super) use super::super::effect::*;
pub(super) use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        definition::BoltDefinition,
        registry::BoltRegistry,
        resources::BoltConfig,
    },
    effect::{
        BoundEffects, EffectKind, EffectNode,
        core::EffectSourceChip,
        effects::shockwave::{ShockwaveMaxRadius, ShockwaveSource, ShockwaveSpeed},
    },
    shared::rng::GameRng,
};

/// Creates a World with `BoltConfig`, `BoltRegistry`, and `GameRng` resources needed for bolt spawning.
pub(super) fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    let mut bolt_registry = BoltRegistry::default();
    bolt_registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_string(),
            base_speed: 720.0,
            min_speed: 360.0,
            max_speed: 1440.0,
            radius: 14.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
        },
    );
    world.insert_resource(bolt_registry);
    world.insert_resource(GameRng::from_seed(42));
    world
}

/// Creates a default config for most tests.
pub(super) fn default_config() -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        bumps_required: 3,
        spawn_count: 1,
        inherit: true,
        shockwave_range: 160.0,
        shockwave_speed: 500.0,
    }
}
