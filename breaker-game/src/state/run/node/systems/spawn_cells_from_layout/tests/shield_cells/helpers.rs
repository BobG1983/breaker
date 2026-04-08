use bevy::prelude::*;

use crate::{
    cells::{
        CellTypeDefinition,
        definition::ShieldBehavior,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::PlayfieldConfig,
    state::run::node::{
        ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned,
        systems::spawn_cells_from_layout::system::spawn_cells_from_layout,
    },
};

/// Creates a registry containing a shield cell type ('H') plus a normal cell ('N').
pub(super) fn shield_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "H".to_owned(),
        CellTypeDefinition {
            id: "shield".to_owned(),
            alias: "H".to_owned(),
            hp: 20.0,
            color_rgb: [0.8, 0.8, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,
            shield: Some(ShieldBehavior {
                count: 3,
                radius: 60.0,
                speed: std::f32::consts::FRAC_PI_2,
                hp: 10.0,
                color_rgb: [0.5, 0.8, 1.0],
            }),
            effects: None,
        },
    );
    registry.insert(
        "N".to_owned(),
        CellTypeDefinition {
            id: "normal".to_owned(),
            alias: "N".to_owned(),
            hp: 1.0,
            color_rgb: [1.0, 0.5, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,
            shield: None,
            effects: None,
        },
    );
    registry
}

pub(super) fn shield_layout() -> NodeLayout {
    NodeLayout {
        name: "shield_test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!["H".to_owned(), "N".to_owned()]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    }
}

pub(super) fn shield_test_app(layout: NodeLayout, registry: CellTypeRegistry) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(registry)
        .add_systems(Startup, spawn_cells_from_layout);
    app
}
