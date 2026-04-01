pub(super) use bevy::prelude::*;
pub(super) use rantzsoft_spatial2d::components::Position2D;

pub(super) use super::super::effect::*;
pub(super) use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        resources::BoltConfig,
    },
    effect::{
        BoundEffects, EffectKind, EffectNode,
        core::EffectSourceChip,
        effects::shockwave::{ShockwaveMaxRadius, ShockwaveSource, ShockwaveSpeed},
    },
    shared::rng::GameRng,
};

/// Creates a World with `BoltConfig` and `GameRng` resources needed for bolt spawning.
pub(super) fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
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
