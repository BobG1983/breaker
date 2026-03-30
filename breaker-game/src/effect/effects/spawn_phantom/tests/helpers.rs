//! Shared test helpers for `SpawnPhantom` effect tests.

use bevy::prelude::*;

use crate::{bolt::resources::BoltConfig, shared::rng::GameRng};

pub(super) fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}
