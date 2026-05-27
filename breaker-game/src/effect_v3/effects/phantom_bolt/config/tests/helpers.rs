//! Shared test fixtures for Wave 4B `SpawnPhantomConfig` tests.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::components::{Bolt, ExtraBolt, PhantomBolt, PhantomDamagedCells, PhantomDedupKey},
    shared::rng::GameRng,
};

pub(super) fn world_with_assets() -> World {
    let mut world = World::new();
    world.insert_resource(GameRng::from_seed(42));
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    world
}

pub(super) fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
    world.spawn((Bolt, Position2D(pos), Velocity2D(vel))).id()
}

/// Pre-spawn a phantom with the given `(chip, fired_from)` dedup key.
pub(super) fn spawn_keyed_phantom(world: &mut World, chip: &str, fired_from: Entity) {
    world.spawn((
        Bolt,
        ExtraBolt,
        PhantomBolt,
        PhantomDedupKey::Chip {
            chip: chip.to_string(),
            fired_from,
        },
        PhantomDamagedCells::default(),
    ));
}
