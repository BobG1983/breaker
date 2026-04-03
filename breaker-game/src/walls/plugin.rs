//! Wall plugin registration.

use bevy::prelude::*;

use crate::{
    state::{run::node::systems::spawn_walls, types::NodeState},
    walls::messages::WallsSpawned,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::types::{AppState, GamePhase, RunPhase};

    #[test]
    fn plugin_builds() {
        let mut registry = crate::walls::WallRegistry::default();
        registry.insert("Wall".to_string(), crate::walls::WallDefinition::default());
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<NodeState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            .insert_resource(registry)
            .add_plugins(WallPlugin)
            .update();
    }
}
