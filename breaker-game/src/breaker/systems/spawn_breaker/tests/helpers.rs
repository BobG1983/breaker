use bevy::prelude::*;

use super::super::system::*;
use crate::{
    breaker::{
        definition::BreakerDefinition, messages::BreakerSpawned, registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    shared::PlayfieldConfig,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();

    // Set up registry with default "Aegis" definition
    let def = BreakerDefinition::default();
    let mut registry = BreakerRegistry::default();
    registry.insert(
        "Aegis".to_string(),
        BreakerDefinition {
            name: "Aegis".to_string(),
            life_pool: Some(3),
            ..def
        },
    );
    app.insert_resource(registry);
    app.insert_resource(SelectedBreaker::default());

    app.add_systems(Startup, spawn_or_reuse_breaker);
    app
}
