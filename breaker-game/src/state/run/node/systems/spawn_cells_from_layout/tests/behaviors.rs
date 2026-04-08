use bevy::prelude::*;

use super::{super::system::spawn_cells_from_layout, helpers::*};
use crate::{
    cells::{
        CellTypeDefinition,
        components::*,
        definition::CellBehavior,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::PlayfieldConfig,
    state::run::{
        definition::NodeType,
        node::{ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned},
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
    },
};

// --- A2: CellBehavior wiring tests ---

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

/// Creates a registry with a regen cell type ('R') and a normal cell type ('N').
fn behavior_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "R".to_owned(),
        CellTypeDefinition {
            id: "regen".to_owned(),
            alias: "R".to_owned(),
            hp: 8.0,
            color_rgb: [0.5, 1.0, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: Some(vec![CellBehavior::Regen { rate: 2.0 }]),
            shield: None,
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

fn behavior_test_app(layout: NodeLayout, registry: CellTypeRegistry) -> App {
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

// NOTE: locked_cell_definition_spawns_with_locked_component,
// non_locked_cell_does_not_have_locked_component, and
// locked_cell_definition_spawns_with_lock_adjacents_component
// have been REMOVED — locking is no longer driven by CellBehavior.

#[test]
fn regen_cell_definition_spawns_with_cell_regen_component() {
    let layout = NodeLayout {
        name: "regen_test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("R"), s("N")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let regen_cells: Vec<&CellRegen> = app
        .world_mut()
        .query::<(&Cell, &CellRegen)>()
        .iter(app.world())
        .map(|(_, regen)| regen)
        .collect();
    assert_eq!(
        regen_cells.len(),
        1,
        "cell with behaviors: [Regen {{ rate: 2.0 }}] should have CellRegen component"
    );
    assert!(
        (regen_cells[0].rate - 2.0).abs() < f32::EPSILON,
        "CellRegen rate should be 2.0, got {}",
        regen_cells[0].rate
    );
}

#[test]
fn non_regen_cell_does_not_have_cell_regen_component() {
    let layout = NodeLayout {
        name: "no_regen_test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("N"), s("N")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let regen_count = app
        .world_mut()
        .query::<(&Cell, &CellRegen)>()
        .iter(app.world())
        .count();
    assert_eq!(
        regen_count, 0,
        "cells with behaviors: None should NOT have CellRegen component"
    );
}

// --- A4: HP multiplier tests ---

#[test]
fn cell_hp_scaled_by_node_assignment_hp_mult() {
    let layout = NodeLayout {
        name: "hp_mult_test".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("S")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .insert_resource(NodeOutcome {
            node_index: 0,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type: NodeType::Active,
                tier_index: 0,
                hp_mult: 3.0,
                timer_mult: 1.0,
            }],
        })
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    // 'S' has hp=1.0, hp_mult=3.0 -> CellHealth { current: 3.0, max: 3.0 }
    let healths: Vec<&CellHealth> = app
        .world_mut()
        .query::<&CellHealth>()
        .iter(app.world())
        .collect();
    assert_eq!(healths.len(), 1);
    assert!(
        (healths[0].current - 3.0).abs() < f32::EPSILON,
        "cell current HP should be 1.0 * 3.0 = 3.0, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].max - 3.0).abs() < f32::EPSILON,
        "cell max HP should be 1.0 * 3.0 = 3.0, got {}",
        healths[0].max
    );
}

#[test]
fn cell_hp_unchanged_when_hp_mult_is_one() {
    let layout = NodeLayout {
        name: "hp_mult_one_test".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![s("T")]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .insert_resource(NodeOutcome {
            node_index: 0,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type: NodeType::Passive,
                tier_index: 0,
                hp_mult: 1.0,
                timer_mult: 1.0,
            }],
        })
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    // 'T' has hp=3.0, hp_mult=1.0 -> CellHealth { current: 3.0, max: 3.0 }
    let healths: Vec<&CellHealth> = app
        .world_mut()
        .query::<&CellHealth>()
        .iter(app.world())
        .collect();
    assert_eq!(healths.len(), 1);
    assert!(
        (healths[0].current - 3.0).abs() < f32::EPSILON,
        "cell current HP should be 3.0 * 1.0 = 3.0, got {}",
        healths[0].current
    );
    assert!(
        (healths[0].max - 3.0).abs() < f32::EPSILON,
        "cell max HP should be 3.0 * 1.0 = 3.0, got {}",
        healths[0].max
    );
}

#[test]
fn cell_spacing_matches_config() {
    let layout = full_layout();
    let config = CellConfig::default();
    let step_x = config.width + config.padding_x;
    let step_y = config.height + config.padding_y;
    let mut app = test_app(layout);
    app.update();

    let positions = collect_sorted_cell_positions(&mut app);

    // Check horizontal spacing within row 0 (first 3 cells)
    let dx_01 = positions[1].0 - positions[0].0;
    assert!(
        (dx_01 - step_x).abs() < 0.01,
        "horizontal spacing should be {step_x}, got {dx_01}"
    );
    let dx_12 = positions[2].0 - positions[1].0;
    assert!(
        (dx_12 - step_x).abs() < 0.01,
        "horizontal spacing should be {step_x}, got {dx_12}"
    );

    // Check vertical spacing between row 0 and row 1 (same column)
    let dy = positions[0].1 - positions[3].1;
    assert!(
        (dy - step_y).abs() < 0.01,
        "vertical spacing should be {step_y}, got {dy}"
    );
}
