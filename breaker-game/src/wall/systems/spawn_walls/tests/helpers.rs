//! Shared test helpers for `spawn_walls` tests.

use bevy::prelude::*;

use super::super::*;
use crate::{
    shared::PlayfieldConfig,
    wall::{definition::WallDefinition, messages::WallsSpawned, registry::WallRegistry},
};

pub(super) fn test_app() -> App {
    let mut registry = WallRegistry::default();
    registry.insert(
        "Wall".to_string(),
        WallDefinition {
            name: "Wall".to_string(),
            ..WallDefinition::default()
        },
    );
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WallsSpawned>()
        .init_resource::<PlayfieldConfig>()
        .insert_resource(registry)
        .add_systems(Update, spawn_walls);
    app
}
