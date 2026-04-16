use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    cells::{
        CellTypeDefinition,
        definition::Toughness,
        resources::{CellConfig, CellTypeRegistry},
    },
    prelude::*,
    state::run::{
        node::{
            ActiveNodeLayout,
            definition::{LockMap, NodePool},
            messages::CellsSpawned,
            systems::spawn_cells_from_layout::system::{
                compute_grid_scale, grid_extent, spawn_cells_from_layout,
            },
        },
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
    },
};

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

pub(super) fn test_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "S".to_owned(),
        CellTypeDefinition {
            id:                "standard".to_owned(),
            alias:             "S".to_owned(),
            toughness:         Toughness::default(),
            color_rgb:         [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,

            effects: None,
        },
    );
    registry.insert(
        "T".to_owned(),
        CellTypeDefinition {
            id:                "tough".to_owned(),
            alias:             "T".to_owned(),
            toughness:         crate::cells::definition::Toughness::Tough,
            color_rgb:         [2.5, 0.2, 4.0],
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

/// A full 3x2 layout with no gaps.
pub(super) fn full_layout() -> NodeLayout {
    NodeLayout {
        name:            "full".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("T"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    }
}

/// A 3x2 layout with gaps (dots).
pub(super) fn sparse_layout() -> NodeLayout {
    NodeLayout {
        name:            "sparse".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("."), s("S"), s(".")], vec![s("T"), s("."), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    }
}

pub(super) fn test_app(layout: NodeLayout) -> App {
    TestAppBuilder::new()
        .with_message::<CellsSpawned>()
        .with_resource::<CellConfig>()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<Assets<Mesh>>()
        .with_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .with_system(Startup, spawn_cells_from_layout)
        .build()
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
        width:     126.0,
        height:    43.0,
        padding_x: 7.0,
        padding_y: 7.0,
    }
}

/// Returns a `PlayfieldConfig` with RON-like values (not Rust `Default`).
pub(super) fn ron_like_playfield_config() -> PlayfieldConfig {
    PlayfieldConfig {
        width:                1440.0,
        height:               1080.0,
        zone_fraction:        0.667,
        wall_thickness:       180.0,
        background_color_rgb: [0.02, 0.01, 0.04],
    }
}

/// Creates a test `App` with explicit RON-like configs for grid-scale tests.
pub(super) fn scaled_test_app(layout: NodeLayout) -> App {
    TestAppBuilder::new()
        .with_message::<CellsSpawned>()
        .insert_resource(ron_like_cell_config())
        .insert_resource(ron_like_playfield_config())
        .with_resource::<Assets<Mesh>>()
        .with_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .with_system(Startup, spawn_cells_from_layout)
        .build()
}

/// Builds a `NodeLayout` filled entirely with "S" cells.
pub(super) fn uniform_layout(cols: u32, rows: u32, grid_top_offset: f32) -> NodeLayout {
    let grid = vec![vec![s("S"); cols as usize]; rows as usize];
    NodeLayout {
        name: format!("uniform_{cols}x{rows}"),
        timer_secs: 60.0,
        cols,
        rows,
        grid_top_offset,
        grid,
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
        sequences: None,
    }
}

// =============================================================================
// Lock Resolution Helpers
// =============================================================================

/// A 3x2 layout with a single lock: cell (0,1) locked by cell (1,0).
pub(super) fn locked_layout_3x2_single() -> NodeLayout {
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 1), vec![(1, 0)]);
    NodeLayout {
        name:            "locked_3x2_single".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S")], vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    }
}

/// A 2x1 layout with cell (0,0) locked by cell (0,1).
pub(super) fn simple_locked_layout() -> NodeLayout {
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    NodeLayout {
        name:            "simple_locked".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    }
}

/// A 3x1 layout with chain: A(0,0) locked by B(0,1), B locked by C(0,2).
pub(super) fn chain_locked_layout() -> NodeLayout {
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 1), vec![(0, 2)]);
    NodeLayout {
        name:            "chain_locked".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    }
}

/// A 2x2 layout with diamond dependency:
/// A=(0,0) locked by B=(0,1) and C=(1,0);
/// B=(0,1) locked by D=(1,1);
/// C=(1,0) locked by D=(1,1).
pub(super) fn diamond_locked_layout() -> NodeLayout {
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1), (1, 0)]);
    locks.insert((0, 1), vec![(1, 1)]);
    locks.insert((1, 0), vec![(1, 1)]);
    NodeLayout {
        name:            "diamond_locked".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            2,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S")], vec![s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    }
}

/// A 2x1 layout with circular lock: A(0,0) locked by B(0,1), B locked by A.
pub(super) fn circular_locked_layout() -> NodeLayout {
    let mut locks: LockMap = HashMap::new();
    locks.insert((0, 0), vec![(0, 1)]);
    locks.insert((0, 1), vec![(0, 0)]);
    NodeLayout {
        name:            "circular_locked".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("S")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           Some(locks),
        sequences:       None,
    }
}

/// Collects Cell entities keyed by their approximate grid position.
pub(super) fn collect_cells_by_grid_position(
    app: &mut App,
    layout: &NodeLayout,
) -> HashMap<(usize, usize), Entity> {
    let config = app.world().resource::<CellConfig>().clone();
    let playfield = app.world().resource::<PlayfieldConfig>().clone();

    let dims = compute_grid_scale(
        &config,
        &playfield,
        layout.cols,
        layout.rows,
        layout.grid_top_offset,
    );

    let grid_width = grid_extent(
        dims.step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        dims.padding_x,
    );
    let start_x = -grid_width / 2.0 + dims.cell_width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - dims.cell_height / 2.0;

    let mut expected_positions: Vec<((usize, usize), (f32, f32))> = Vec::new();
    for (row_idx, row) in layout.grid.iter().enumerate() {
        for (col_idx, alias) in row.iter().enumerate() {
            if alias == "." {
                continue;
            }
            let col_f = f32::from(u16::try_from(col_idx).unwrap_or(u16::MAX));
            let row_f = f32::from(u16::try_from(row_idx).unwrap_or(u16::MAX));
            let x = col_f.mul_add(dims.step_x, start_x);
            let y = row_f.mul_add(-dims.step_y, start_y);
            expected_positions.push(((row_idx, col_idx), (x, y)));
        }
    }

    let cell_entities: Vec<(Entity, Vec2)> = app
        .world_mut()
        .query_filtered::<(Entity, &Position2D), With<Cell>>()
        .iter(app.world())
        .map(|(e, pos)| (e, pos.0))
        .collect();

    let mut result = HashMap::new();
    for ((row, col), (ex, ey)) in &expected_positions {
        for &(entity, pos) in &cell_entities {
            if (pos.x - ex).abs() < 0.01 && (pos.y - ey).abs() < 0.01 {
                result.insert((*row, *col), entity);
                break;
            }
        }
    }
    result
}

/// Creates a test `App` with `NodeOutcome` and `NodeSequence` resources.
pub(super) fn test_app_with_sequence(layout: NodeLayout) -> App {
    TestAppBuilder::new()
        .with_message::<CellsSpawned>()
        .with_resource::<CellConfig>()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<Assets<Mesh>>()
        .with_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(test_registry())
        .insert_resource(NodeOutcome {
            node_index: 0,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type:  NodeType::Active,
                tier_index: 0,
                timer_mult: 1.0,
            }],
        })
        .with_system(Startup, spawn_cells_from_layout)
        .build()
}

/// Creates a test `App` with `ToughnessConfig`, `NodeOutcome` (tier/position),
/// and `NodeSequence` for toughness-based HP testing.
///
/// `tier` and `position_in_tier` are set on `NodeOutcome`. If `is_boss` is true,
/// the single `NodeAssignment` uses `NodeType::Boss`.
pub(super) fn test_app_with_toughness(
    layout: NodeLayout,
    registry: CellTypeRegistry,
    tier: u32,
    position_in_tier: u32,
    is_boss: bool,
) -> App {
    use crate::cells::resources::ToughnessConfig;

    TestAppBuilder::new()
        .with_message::<CellsSpawned>()
        .with_resource::<CellConfig>()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<Assets<Mesh>>()
        .with_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(registry)
        .insert_resource(ToughnessConfig::default())
        .insert_resource(NodeOutcome {
            node_index: 0,
            tier,
            position_in_tier,
            ..Default::default()
        })
        .insert_resource(NodeSequence {
            assignments: vec![NodeAssignment {
                node_type:  if is_boss {
                    NodeType::Boss
                } else {
                    NodeType::Active
                },
                tier_index: tier,
                timer_mult: 1.0,
            }],
        })
        .with_system(Startup, spawn_cells_from_layout)
        .build()
}

/// Creates a test `App` with a registry containing an "F" type
/// (`required_to_clear=false`) plus the standard "S" type.
pub(super) fn test_app_with_non_required_cell(layout: NodeLayout) -> App {
    let mut registry = test_registry();
    registry.insert(
        "F".to_owned(),
        CellTypeDefinition {
            id:                "filler".to_owned(),
            alias:             "F".to_owned(),
            toughness:         Toughness::default(),
            color_rgb:         [0.5, 0.5, 0.5],
            required_to_clear: false,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,

            effects: None,
        },
    );

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
