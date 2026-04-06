use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::{
        CellTypeDefinition,
        components::Cell,
        definition::CellBehavior,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::PlayfieldConfig,
    state::run::node::{
        ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned,
        systems::spawn_cells_from_layout::system::spawn_cells_from_layout,
    },
};

pub(super) fn test_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        'S',
        CellTypeDefinition {
            id: "standard".to_owned(),
            alias: 'S',
            hp: 1.0,
            color_rgb: [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
            effects: None,
        },
    );
    registry.insert(
        'T',
        CellTypeDefinition {
            id: "tough".to_owned(),
            alias: 'T',
            hp: 3.0,
            color_rgb: [2.5, 0.2, 4.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
            effects: None,
        },
    );
    registry
}

/// A full 3x2 layout with no gaps.
pub(super) fn full_layout() -> NodeLayout {
    NodeLayout {
        name: "full".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec!['T', 'S', 'S'], vec!['S', 'S', 'S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    }
}

/// A 3x2 layout with gaps (dots).
pub(super) fn sparse_layout() -> NodeLayout {
    NodeLayout {
        name: "sparse".to_owned(),
        timer_secs: 60.0,
        cols: 3,
        rows: 2,
        grid_top_offset: 50.0,
        grid: vec![vec!['.', 'S', '.'], vec!['T', '.', 'S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    }
}

pub(super) fn test_app(layout: NodeLayout) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .add_systems(Startup, spawn_cells_from_layout);
    app
}

pub(super) fn collect_sorted_cell_positions(app: &mut App) -> Vec<(f32, f32)> {
    let mut positions: Vec<(f32, f32)> = app
        .world_mut()
        .query_filtered::<&Position2D, With<Cell>>()
        .iter(app.world())
        .map(|pos| (pos.0.x, pos.0.y))
        .collect();
    positions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
    positions
}

pub(super) fn assert_positions_match(actual: &[(f32, f32)], expected: &[(f32, f32)]) {
    assert_eq!(actual.len(), expected.len(), "position count mismatch");
    for (i, ((ax, ay), (ex, ey))) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (ax - ex).abs() < 0.01,
            "cell {i} x: expected {ex:.2}, got {ax:.2}"
        );
        assert!(
            (ay - ey).abs() < 0.01,
            "cell {i} y: expected {ey:.2}, got {ay:.2}"
        );
    }
}

/// Returns a `CellConfig` with RON-like values (not Rust `Default`).
pub(super) fn ron_like_cell_config() -> CellConfig {
    CellConfig {
        width: 126.0,
        height: 43.0,
        padding_x: 7.0,
        padding_y: 7.0,
    }
}

/// Returns a `PlayfieldConfig` with RON-like values (not Rust `Default`).
pub(super) fn ron_like_playfield_config() -> PlayfieldConfig {
    PlayfieldConfig {
        width: 1440.0,
        height: 1080.0,
        zone_fraction: 0.667,
        wall_thickness: 180.0,
        background_color_rgb: [0.02, 0.01, 0.04],
    }
}

/// Creates a test `App` with explicit RON-like configs for grid-scale tests.
pub(super) fn scaled_test_app(layout: NodeLayout) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .insert_resource(ron_like_cell_config())
        .insert_resource(ron_like_playfield_config())
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .add_systems(Startup, spawn_cells_from_layout);
    app
}

/// Builds a `NodeLayout` filled entirely with 'S' cells.
pub(super) fn uniform_layout(cols: u32, rows: u32, grid_top_offset: f32) -> NodeLayout {
    let grid = vec![vec!['S'; cols as usize]; rows as usize];
    NodeLayout {
        name: format!("uniform_{cols}x{rows}"),
        timer_secs: 60.0,
        cols,
        rows,
        grid_top_offset,
        grid,
        pool: NodePool::default(),
        entity_scale: 1.0,
    }
}
