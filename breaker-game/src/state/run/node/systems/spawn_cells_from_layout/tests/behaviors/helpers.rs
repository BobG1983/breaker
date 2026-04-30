use bevy::prelude::*;

use super::super::super::spawning::spawn_cells_from_layout;
use crate::{
    cells::{
        CellTypeDefinition,
        definition::{CellBehavior, Toughness},
        resources::{CellConfig, CellTypeRegistry},
    },
    prelude::*,
    state::run::node::{ActiveNodeLayout, messages::CellsSpawned},
};

/// Helper to reduce verbosity of String grid construction.
pub(super) fn s(val: &str) -> String {
    val.to_owned()
}

/// Creates a registry with a regen cell type ('R') and a normal cell type ('N').
pub(super) fn behavior_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "R".to_owned(),
        CellTypeDefinition {
            id:                "regen".to_owned(),
            alias:             "R".to_owned(),
            toughness:         Toughness::default(),
            color_rgb:         [0.5, 1.0, 0.5],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         Some(vec![CellBehavior::Regen { rate: 2.0 }]),

            effects: None,
        },
    );
    registry.insert(
        "N".to_owned(),
        CellTypeDefinition {
            id:                "normal".to_owned(),
            alias:             "N".to_owned(),
            toughness:         Toughness::default(),
            color_rgb:         [1.0, 0.5, 0.5],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,

            effects: None,
        },
    );
    registry
}

pub(super) fn behavior_test_app(layout: NodeLayout, registry: CellTypeRegistry) -> App {
    TestAppBuilder::new()
        .with_message::<CellsSpawned>()
        .with_resource::<CellConfig>()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<Assets<Mesh>>()
        .with_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(registry)
        .with_system(Startup, spawn_cells_from_layout)
        .build()
}
