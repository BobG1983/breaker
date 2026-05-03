//! Shared test helpers for `tick_phantom_breaker_lifespan` system tests.

use bevy::prelude::*;

use super::super::tick_phantom_breaker_lifespan;
use crate::{
    breaker::{
        builder::core::types::BreakerPhantomParams, components::Breaker,
        test_utils::default_breaker_definition,
    },
    prelude::*,
};

/// Phantom params used consistently across tests.
pub(super) fn test_phantom_params(lifespan_secs: f32) -> BreakerPhantomParams {
    BreakerPhantomParams {
        lifespan:          lifespan_secs,
        phantom_color_rgb: [0.4, 0.8, 1.0],
        flicker_frequency: 4.0,
        flicker_min_alpha: 0.3,
    }
}

/// Spawns a headless extra phantom breaker with the given lifespan.
///
/// Uses `world.commands()` + `world.flush()` for immediate application.
/// Returns the spawned `Entity`.
pub(super) fn spawn_phantom(app: &mut App, lifespan_secs: f32) -> Entity {
    let def = default_breaker_definition();
    let params = test_phantom_params(lifespan_secs);
    let world = app.world_mut();
    let entity = Breaker::builder()
        .definition(&def)
        .phantom(params)
        .headless()
        .extra()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

/// Builds a minimal `App` with `tick_phantom_breaker_lifespan` in `FixedUpdate`
/// and `DespawnEntity` message capture. No `RantzDmgPlugin` — proves the system
/// only emits; does not despawn directly.
pub(super) fn lifespan_emit_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DespawnEntity>()
        .with_system(FixedUpdate, tick_phantom_breaker_lifespan)
        .build()
}

/// Builds an end-to-end `App` with `RantzDmgPlugin` + `tick_phantom_breaker_lifespan`
/// registered in `FixedUpdate`. `DespawnEntity` is automatically registered by
/// `RantzDmgPlugin`; we attach a collector so tests can inspect the message.
pub(super) fn lifespan_despawn_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .with_system(FixedUpdate, tick_phantom_breaker_lifespan)
        .build();
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

/// The fixed timestep delta used in concrete numeric assertions.
/// `Time<Fixed>` defaults to `1.0 / 64.0`.
pub(super) const FIXED_DT: f32 = 1.0 / 64.0;
