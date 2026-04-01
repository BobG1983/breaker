use bevy::prelude::*;

use crate::{
    bolt::{
        definition::BoltDefinition, messages::BoltSpawned, registry::BoltRegistry,
        resources::BoltConfig,
    },
    breaker::{
        BreakerConfig,
        definition::{BreakerDefinition, BreakerStatOverrides},
        registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    run::RunState,
    shared::GameRng,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltSpawned>()
        .init_resource::<BoltConfig>()
        .init_resource::<BreakerConfig>()
        .init_resource::<RunState>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<GameRng>();

    // Set up registries with the default bolt definition
    let bolt_def = make_default_bolt_definition();
    let mut bolt_registry = BoltRegistry::default();
    bolt_registry.insert("Bolt".to_string(), bolt_def);
    app.insert_resource(bolt_registry);

    let breaker_def = make_default_breaker_definition();
    let mut breaker_registry = BreakerRegistry::default();
    breaker_registry.insert("Aegis".to_string(), breaker_def);
    app.insert_resource(breaker_registry);

    app.insert_resource(SelectedBreaker::default());
    app
}

/// Creates a test app with `BoltRegistry`, `BreakerRegistry`, `SelectedBreaker`,
/// and `GameRng` resources for migration tests that verify the system reads
/// from registries instead of `BoltConfig`.
pub(super) fn test_app_with_registries() -> App {
    test_app()
}

pub(super) fn make_default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
    }
}

pub(super) fn make_default_breaker_definition() -> BreakerDefinition {
    BreakerDefinition {
        name: "Aegis".to_string(),
        bolt: "Bolt".to_string(),
        stat_overrides: BreakerStatOverrides::default(),
        life_pool: Some(3),
        effects: vec![],
    }
}
