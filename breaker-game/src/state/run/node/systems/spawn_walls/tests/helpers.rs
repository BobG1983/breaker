//! Shared test helpers for `spawn_walls` tests.

use bevy::prelude::*;

use crate::{
    shared::PlayfieldConfig,
    state::run::node::systems::spawn_walls::*,
    walls::{definition::WallDefinition, messages::WallsSpawned, registry::WallRegistry},
};

pub(super) fn test_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;
    let mut registry = WallRegistry::default();
    registry.insert(
        "Wall".to_string(),
        WallDefinition {
            name: "Wall".to_string(),
            ..WallDefinition::default()
        },
    );
    TestAppBuilder::new()
        .with_message::<WallsSpawned>()
        .with_resource::<PlayfieldConfig>()
        .insert_resource(registry)
        .with_system(Update, spawn_walls)
        .build()
}
