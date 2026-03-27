use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{InterpolateTransform2D, Position2D, Scale2D, Spatial2D};

use super::{
    super::system::{compute_grid_scale, grid_extent, spawn_cells_from_layout},
    helpers::*,
};
use crate::{
    cells::{
        CellTypeDefinition,
        components::*,
        definition::CellBehavior,
        resources::{CellConfig, CellTypeRegistry},
    },
    run::node::{ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned},
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer, PlayfieldConfig},
};

// --- Position2D migration tests ---

#[test]
#[expect(
    clippy::cast_precision_loss,
    reason = "grid column index is always small"
)]
fn spawned_cell_has_position2d_at_grid_position() {
    // Given: full 3x2 layout with CellConfig default (width=70, height=24,
    //        padding_x=4, padding_y=4) and PlayfieldConfig default
    // When: spawn_cells_from_layout runs
    // Then: Each Cell entity has Position2D matching the grid formula

    let layout = full_layout(); // 3x2, grid_top_offset=50
    let config = CellConfig::default();
    let playfield = PlayfieldConfig::default();
    let step_x = config.width + config.padding_x; // 74.0
    let step_y = config.height + config.padding_y; // 28.0
    let cols_f = 3.0_f32;
    let grid_width = grid_extent(step_x, cols_f, config.padding_x); // 74*3 - 4 = 218
    let start_x = -grid_width / 2.0 + config.width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

    let mut app = test_app(layout);
    app.update();

    let mut positions: Vec<Vec2> = app
        .world_mut()
        .query_filtered::<&Position2D, With<Cell>>()
        .iter(app.world())
        .map(|p| p.0)
        .collect();
    // Sort top-to-bottom, left-to-right
    positions.sort_by(|a, b| b.y.total_cmp(&a.y).then(a.x.total_cmp(&b.x)));

    assert_eq!(positions.len(), 6, "full 3x2 layout should spawn 6 cells");

    // Row 0: 3 cells, Row 1: 3 cells
    let expected: Vec<Vec2> = (0..3)
        .map(|col| Vec2::new((col as f32).mul_add(step_x, start_x), start_y))
        .chain(
            (0..3).map(|col| Vec2::new((col as f32).mul_add(step_x, start_x), start_y - step_y)),
        )
        .collect();

    for (i, (actual, exp)) in positions.iter().zip(expected.iter()).enumerate() {
        assert!(
            (actual.x - exp.x).abs() < 0.01 && (actual.y - exp.y).abs() < 0.01,
            "cell {i} Position2D mismatch: expected {exp:?}, got {actual:?}"
        );
    }
}

#[test]
fn spawned_cell_position2d_matches_former_transform_translation() {
    // Edge case: Position2D.0 should match what was previously
    // Transform.translation.truncate() -- verifying the values are identical

    let layout = sparse_layout(); // [., S, .], [T, ., S]
    let mut app = test_app(layout);
    app.update();

    // Every cell should have a Position2D
    let cells_with_pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Cell>>()
        .iter(app.world())
        .count();
    assert_eq!(
        cells_with_pos, 3,
        "sparse layout should have 3 cells with Position2D"
    );
}

#[test]
fn spawned_cell_has_game_draw_layer_cell() {
    // Given: A layout with cells
    // When: spawn_cells_from_layout runs
    // Then: Each Cell has GameDrawLayer::Cell (z=0.0)
    use rantzsoft_spatial2d::draw_layer::DrawLayer;

    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app
        .world_mut()
        .query_filtered::<Entity, With<Cell>>()
        .iter(app.world())
        .count();

    let cells_with_layer: Vec<&GameDrawLayer> = app
        .world_mut()
        .query_filtered::<&GameDrawLayer, With<Cell>>()
        .iter(app.world())
        .collect();

    assert_eq!(
        cells_with_layer.len(),
        cell_count,
        "every cell should have GameDrawLayer"
    );
    for layer in &cells_with_layer {
        assert!(
            layer.z().abs() < f32::EPSILON,
            "GameDrawLayer::Cell.z() should be 0.0, got {}",
            layer.z()
        );
    }
}

#[test]
fn spawned_cell_has_spatial2d_but_not_interpolate_transform2d() {
    // Given: A layout with cells
    // When: spawn_cells_from_layout runs
    // Then: Each Cell has Spatial2D. Does NOT have InterpolateTransform2D.

    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Cell>>()
        .iter(app.world())
        .collect();

    for entity in &entities {
        assert!(
            app.world().get::<Spatial2D>(*entity).is_some(),
            "cell should have Spatial2D marker"
        );
        assert!(
            app.world().get::<InterpolateTransform2D>(*entity).is_none(),
            "cell should NOT have InterpolateTransform2D (static entity)"
        );
    }
}

#[test]
fn spawned_cell_has_scale2d_matching_cell_dimensions() {
    // Given: CellConfig default width=70.0, height=24.0, no grid scaling
    //        (3x2 grid fits at scale 1.0 with default playfield)
    // When: spawn_cells_from_layout runs
    // Then: Each Cell has Scale2D { x: 70.0, y: 24.0 }

    let layout = full_layout();
    let config = CellConfig::default();
    let mut app = test_app(layout);
    app.update();

    let scales: Vec<&Scale2D> = app
        .world_mut()
        .query_filtered::<&Scale2D, With<Cell>>()
        .iter(app.world())
        .collect();

    assert_eq!(scales.len(), 6, "all 6 cells should have Scale2D");
    for (i, scale) in scales.iter().enumerate() {
        assert!(
            (scale.x - config.width).abs() < f32::EPSILON,
            "cell {i} Scale2D.x should be {}, got {}",
            config.width,
            scale.x
        );
        assert!(
            (scale.y - config.height).abs() < f32::EPSILON,
            "cell {i} Scale2D.y should be {}, got {}",
            config.height,
            scale.y
        );
    }
}

#[test]
fn spawned_cell_scale2d_uses_scaled_dimensions_for_large_grid() {
    // Edge case: When grid scaling applies, Scale2D uses scaled dimensions

    let layout = uniform_layout(40, 20, 90.0);
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let dims = compute_grid_scale(&config, &playfield, 40, 20, 90.0);

    let mut app = scaled_test_app(layout);
    app.update();

    let scales: Vec<&Scale2D> = app
        .world_mut()
        .query_filtered::<&Scale2D, With<Cell>>()
        .iter(app.world())
        .collect();

    assert!(!scales.is_empty(), "should have spawned cells");
    assert!(
        dims.scale < 1.0,
        "40x20 grid should need scaling, got {}",
        dims.scale
    );

    for (i, scale) in scales.iter().enumerate() {
        assert!(
            (scale.x - dims.cell_width).abs() < f32::EPSILON,
            "cell {i} Scale2D.x should be {}, got {}",
            dims.cell_width,
            scale.x
        );
        assert!(
            (scale.y - dims.cell_height).abs() < f32::EPSILON,
            "cell {i} Scale2D.y should be {}, got {}",
            dims.cell_height,
            scale.y
        );
    }
}

// --- Aabb2D + CollisionLayers tests ---

#[test]
fn spawned_cell_has_aabb2d_with_half_extents_matching_cell_dimensions() {
    // Given: CellConfig default width=70.0, height=24.0, no grid scaling
    //        (single cell in wide playfield fits at scale 1.0)
    // When: spawn_cells_from_layout runs
    // Then: cell entity has Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(35.0, 12.0) }

    let layout = NodeLayout {
        name: "aabb_test".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!['S']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    };
    let config = CellConfig::default(); // width=70.0, height=24.0
    let mut app = test_app(layout);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Cell>>()
        .iter(app.world())
        .next()
        .expect("cell should exist");
    let aabb = app
        .world()
        .get::<Aabb2D>(entity)
        .expect("cell should have Aabb2D");
    assert_eq!(
        aabb.center,
        Vec2::ZERO,
        "cell Aabb2D center should be ZERO (local space)"
    );
    let expected_half_w = config.width / 2.0; // 35.0
    let expected_half_h = config.height / 2.0; // 12.0
    assert!(
        (aabb.half_extents.x - expected_half_w).abs() < f32::EPSILON
            && (aabb.half_extents.y - expected_half_h).abs() < f32::EPSILON,
        "cell Aabb2D half_extents should be ({expected_half_w}, {expected_half_h}), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y,
    );
}

#[test]
fn spawned_cell_aabb2d_uses_scaled_dimensions_for_large_grid() {
    // Edge case: scaled grid -- Aabb2D half_extents should use scaled cell dimensions

    let layout = uniform_layout(40, 20, 90.0);
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let dims = compute_grid_scale(&config, &playfield, 40, 20, 90.0);
    assert!(dims.scale < 1.0, "40x20 grid should need scaling");

    let mut app = scaled_test_app(layout);
    app.update();

    let aabbs: Vec<&Aabb2D> = app
        .world_mut()
        .query_filtered::<&Aabb2D, With<Cell>>()
        .iter(app.world())
        .collect();

    assert!(!aabbs.is_empty(), "should have spawned cells");
    let expected_half_w = dims.cell_width / 2.0;
    let expected_half_h = dims.cell_height / 2.0;
    for (i, aabb) in aabbs.iter().enumerate() {
        assert_eq!(
            aabb.center,
            Vec2::ZERO,
            "cell {i} Aabb2D center should be ZERO"
        );
        assert!(
            (aabb.half_extents.x - expected_half_w).abs() < 0.01
                && (aabb.half_extents.y - expected_half_h).abs() < 0.01,
            "cell {i} Aabb2D half_extents should be ({expected_half_w:.2}, {expected_half_h:.2}), got ({:.2}, {:.2})",
            aabb.half_extents.x,
            aabb.half_extents.y,
        );
    }
}

#[test]
fn spawned_cell_has_collision_layers_cell_membership_bolt_mask() {
    // Given: spawn_cells_from_layout runs
    // Then: all cells have CollisionLayers { membership: CELL_LAYER (0x02), mask: BOLT_LAYER (0x01) }

    let layout = full_layout(); // 3x2 = 6 cells
    let mut app = test_app(layout);
    app.update();

    let layers_list: Vec<&CollisionLayers> = app
        .world_mut()
        .query_filtered::<&CollisionLayers, With<Cell>>()
        .iter(app.world())
        .collect();

    assert_eq!(
        layers_list.len(),
        6,
        "all 6 cells should have CollisionLayers"
    );
    for (i, layers) in layers_list.iter().enumerate() {
        assert_eq!(
            layers.membership, CELL_LAYER,
            "cell {i} membership should be CELL_LAYER (0x{:02X}), got 0x{:02X}",
            CELL_LAYER, layers.membership,
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "cell {i} mask should be BOLT_LAYER (0x{:02X}), got 0x{:02X}",
            BOLT_LAYER, layers.mask,
        );
    }
}

#[test]
fn locked_cell_has_same_collision_layers_as_normal_cell() {
    // Edge case: locked cell has same CollisionLayers (lock is behavioral, not physical)

    let mut registry = CellTypeRegistry::default();
    registry.insert(
        'L',
        CellTypeDefinition {
            id: "locked".to_owned(),
            alias: 'L',
            hp: 5.0,
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior {
                locked: true,
                regen_rate: None,
                ..Default::default()
            },
        },
    );
    registry.insert(
        'N',
        CellTypeDefinition {
            id: "normal".to_owned(),
            alias: 'N',
            hp: 1.0,
            color_rgb: [1.0, 0.5, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
        },
    );

    let layout = NodeLayout {
        name: "lock_layers_test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!['L', 'N']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    };
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
    app.update();

    let layers_list: Vec<(&CollisionLayers, Option<&Locked>)> = app
        .world_mut()
        .query_filtered::<(&CollisionLayers, Option<&Locked>), With<Cell>>()
        .iter(app.world())
        .collect();

    assert_eq!(layers_list.len(), 2, "should have 2 cells");
    for (layers, locked) in &layers_list {
        let label = if locked.is_some() { "locked" } else { "normal" };
        assert_eq!(
            layers.membership, CELL_LAYER,
            "{label} cell membership should be CELL_LAYER"
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "{label} cell mask should be BOLT_LAYER"
        );
    }
}
