use bevy::prelude::*;

use super::super::system::setup_run;
use crate::{
    bolt::{definition::BoltDefinition, messages::BoltSpawned, registry::BoltRegistry},
    breaker::{
        definition::BreakerDefinition, messages::BreakerSpawned, registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    shared::{PlayfieldConfig, rng::GameRng},
    state::run::RunState,
};

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
        min_radius: None,
        max_radius: None,
    }
}

pub(super) fn make_aegis_breaker_definition() -> BreakerDefinition {
    ron::de::from_str(r#"(name: "Aegis", life_pool: Some(3), effects: [])"#)
        .expect("test RON should parse")
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .add_message::<BoltSpawned>()
        .init_resource::<RunState>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<GameRng>()
        .init_resource::<PlayfieldConfig>();

    // Set up breaker registry with "Aegis" definition
    let breaker_def = make_aegis_breaker_definition();
    let mut breaker_registry = BreakerRegistry::default();
    breaker_registry.insert("Aegis".to_string(), breaker_def);
    app.insert_resource(breaker_registry);

    // Set up bolt registry with "Bolt" definition
    let bolt_def = make_default_bolt_definition();
    let mut bolt_registry = BoltRegistry::default();
    bolt_registry.insert("Bolt".to_string(), bolt_def);
    app.insert_resource(bolt_registry);

    // Selected breaker defaults to "Aegis"
    app.insert_resource(SelectedBreaker::default());

    app.add_systems(Startup, setup_run);
    app
}
