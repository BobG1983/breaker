//! Wall entity spawning — creates invisible boundary entities for CCD collision.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    shared::PlayfieldConfig,
    wall::{components::Wall, messages::WallsSpawned, registry::WallRegistry},
};

/// Spawns left, right, and ceiling wall entities.
///
/// No floor wall — bolt-lost handles that case separately.
/// Wall thickness is loaded from the `"Wall"` definition in [`WallRegistry`].
pub(crate) fn spawn_walls(
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    wall_registry: Res<WallRegistry>,
    mut walls_spawned: MessageWriter<WallsSpawned>,
) {
    let Some(def) = wall_registry.get("Wall") else {
        warn!("'Wall' definition not found in WallRegistry");
        return;
    };

    // Left wall
    Wall::builder()
        .left(&playfield)
        .definition(def)
        .spawn(&mut commands);

    // Right wall
    Wall::builder()
        .right(&playfield)
        .definition(def)
        .spawn(&mut commands);

    // Ceiling
    Wall::builder()
        .ceiling(&playfield)
        .definition(def)
        .spawn(&mut commands);

    walls_spawned.write(WallsSpawned);
}
