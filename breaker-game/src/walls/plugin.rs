//! Wall plugin registration.

use bevy::prelude::*;

use crate::{
    shared::GameState, state::run::node::systems::spawn_walls, walls::messages::WallsSpawned,
};

/// Plugin for the wall domain.
///
/// Owns wall entities (left, right, ceiling boundaries).
/// Spawns walls on node entry.
pub(crate) struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WallsSpawned>()
            .add_systems(OnEnter(GameState::Playing), spawn_walls);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::GameState;

    #[test]
    fn plugin_builds() {
        let mut registry = crate::walls::WallRegistry::default();
        registry.insert("Wall".to_string(), crate::walls::WallDefinition::default());
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .init_resource::<crate::shared::PlayfieldConfig>()
            .insert_resource(registry)
            .add_plugins(WallPlugin)
            .update();
    }
}
