//! Wall plugin registration.

use bevy::prelude::*;

use crate::{prelude::*, state::run::node::systems::spawn_walls, walls::messages::WallsSpawned};

/// Plugin for the wall domain.
///
/// Owns wall entities (left, right, ceiling boundaries).
/// Spawns walls on node entry.
pub(crate) struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WallsSpawned>()
            .add_systems(OnEnter(NodeState::Loading), spawn_walls);
    }
}
