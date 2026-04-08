use bevy::prelude::*;
use rantzsoft_spatial2d::components::Scale2D;

use super::{super::system::compute_grid_scale, helpers::*};
use crate::{
    cells::components::*,
    state::run::node::{NodeLayout, definition::NodePool},
};

// --- A: Pure function tests for compute_grid_scale ---

#[test]
fn small_grid_returns_scale_one() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 3, 2, 50.0);
    assert!(
        (result.scale - 1.0).abs() < f32::EPSILON,
        "3x2 grid should fit at scale 1.0, got {}",
        result.scale
    );
}

#[test]
fn wide_grid_is_width_constrained() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 30, 2, 90.0);
    // default_grid_width = 30 * 133 - 7 = 3983
    // scale = 1440 / 3983 ~ 0.3615
    let expected = 1440.0 / 3983.0;
    assert!(
        result.scale < 1.0,
        "30-col grid should need scaling, got {}",
        result.scale
    );
    assert!(
        (result.scale - expected).abs() < 0.001,
        "expected scale ~{expected:.4}, got {:.4}",
        result.scale
    );
}

#[test]
fn tall_grid_is_height_constrained() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 3, 30, 90.0);
    // cell_zone_height = 1080 * 0.667 = 720.36
    // available_height = 720.36 - 90.0 = 630.36
    // default_grid_height = 30 * 50 - 7 = 1493
    // scale = 630.36 / 1493 ~ 0.4222
    let expected = 630.36 / 1493.0;
    assert!(
        result.scale < 1.0,
        "30-row grid should need scaling, got {}",
        result.scale
    );
    assert!(
        (result.scale - expected).abs() < 0.001,
        "expected scale ~{expected:.4}, got {:.4}",
        result.scale
    );
}

#[test]
fn scale_capped_at_one_for_tiny_grid() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 1, 1, 50.0);
    assert!(
        (result.scale - 1.0).abs() < f32::EPSILON,
        "1x1 grid should be scale 1.0, got {}",
        result.scale
    );
}

#[test]
fn corridor_layout_ten_by_five_returns_scale_one() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 10, 5, 90.0);
    // grid_width = 10*133 - 7 = 1323 < 1440
    // grid_height = 5*50 - 7 = 243 < 630.36
    assert!(
        (result.scale - 1.0).abs() < f32::EPSILON,
        "10x5 grid should fit at scale 1.0, got {}",
        result.scale
    );
}

#[test]
fn extreme_grid_128x128_produces_positive_sub_unit_scale() {
    let config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let result = compute_grid_scale(&config, &playfield, 128, 128, 90.0);
    // default_grid_width = 128*133 - 7 = 17017
    // scale_x = 1440 / 17017 ~ 0.0846
    // default_grid_height = 128*50 - 7 = 6393
    // scale_y = 630.36 / 6393 ~ 0.0986
    // scale = min(0.0846, 0.0986) ~ 0.0846
    let expected = 1440.0 / 17017.0;
    assert!(
        result.scale > 0.0,
        "scale must be positive, got {}",
        result.scale
    );
    assert!(
        result.scale < 1.0,
        "128x128 grid must scale down, got {}",
        result.scale
    );
    assert!(
        (result.scale - expected).abs() < 0.001,
        "expected scale ~{expected:.4}, got {:.4}",
        result.scale
    );
}

// --- B: Integration tests for scaled cell spawning ---

#[test]
fn large_grid_cells_have_scaled_dimensions() {
    let layout = uniform_layout(40, 20, 90.0);
    let mut app = scaled_test_app(layout);
    app.update();

    let widths: Vec<f32> = app
        .world_mut()
        .query::<(&Cell, &CellWidth)>()
        .iter(app.world())
        .map(|(_, w)| w.value)
        .collect();
    let heights: Vec<f32> = app
        .world_mut()
        .query::<(&Cell, &CellHeight)>()
        .iter(app.world())
        .map(|(_, h)| h.value)
        .collect();

    assert!(!widths.is_empty(), "should have spawned cells");

    // All widths should be less than the base 126.0 (grid is too wide)
    for (i, &w) in widths.iter().enumerate() {
        assert!(
            w < 126.0,
            "cell {i} CellWidth={w} should be < 126.0 for a 40x20 grid"
        );
    }
    // All heights should be less than the base 43.0
    for (i, &h) in heights.iter().enumerate() {
        assert!(
            h < 43.0,
            "cell {i} CellHeight={h} should be < 43.0 for a 40x20 grid"
        );
    }

    // All widths should be uniform
    let first_w = widths[0];
    for (i, &w) in widths.iter().enumerate() {
        assert!(
            (w - first_w).abs() < f32::EPSILON,
            "cell {i} CellWidth={w} differs from first={first_w}"
        );
    }
    // All heights should be uniform
    let first_h = heights[0];
    for (i, &h) in heights.iter().enumerate() {
        assert!(
            (h - first_h).abs() < f32::EPSILON,
            "cell {i} CellHeight={h} differs from first={first_h}"
        );
    }
}

#[test]
fn large_grid_cells_within_cell_zone_bounds() {
    let layout = uniform_layout(40, 20, 90.0);
    let playfield = ron_like_playfield_config();
    let cell_zone_height = playfield.height * playfield.zone_fraction; // 720.36
    let zone_bottom = playfield.top() - cell_zone_height; // 540.0 - 720.36 = -180.36
    let mut app = scaled_test_app(layout);
    app.update();

    let positions = collect_sorted_cell_positions(&mut app);

    for &(x, y) in &positions {
        assert!(
            y > zone_bottom,
            "cell y={y} below cell zone bottom {zone_bottom}"
        );
        assert!(
            y < playfield.top(),
            "cell y={y} above playfield top {}",
            playfield.top()
        );
        assert!(
            x.abs() < playfield.right(),
            "cell |x|={} outside playfield right {}",
            x.abs(),
            playfield.right()
        );
    }
}

#[test]
fn small_grid_preserves_original_dimensions() {
    let layout = uniform_layout(3, 2, 50.0);
    let mut app = scaled_test_app(layout);
    app.update();

    for (_, w) in app
        .world_mut()
        .query::<(&Cell, &CellWidth)>()
        .iter(app.world())
    {
        assert!(
            (w.value - 126.0).abs() < f32::EPSILON,
            "3x2 grid CellWidth should be 126.0, got {}",
            w.value
        );
    }
    for (_, h) in app
        .world_mut()
        .query::<(&Cell, &CellHeight)>()
        .iter(app.world())
    {
        assert!(
            (h.value - 43.0).abs() < f32::EPSILON,
            "3x2 grid CellHeight should be 43.0, got {}",
            h.value
        );
    }
}

#[test]
fn large_grid_transform_scale_matches_cell_dimensions() {
    let layout = uniform_layout(40, 20, 90.0);
    let mut app = scaled_test_app(layout);
    app.update();

    for (_, w, h, scale) in app
        .world_mut()
        .query::<(&Cell, &CellWidth, &CellHeight, &Scale2D)>()
        .iter(app.world())
    {
        assert!(
            (scale.x - w.value).abs() < f32::EPSILON,
            "Scale2D.x={} should match CellWidth={}",
            scale.x,
            w.value
        );
        assert!(
            (scale.y - h.value).abs() < f32::EPSILON,
            "Scale2D.y={} should match CellHeight={}",
            scale.y,
            h.value
        );
    }
}

#[test]
fn single_cell_grid_spawns_centered_at_full_scale() {
    let layout = NodeLayout {
        name: "single".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!["S".to_owned()]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
    };
    let mut app = scaled_test_app(layout);
    app.update();

    let cells: Vec<(
        &CellWidth,
        &CellHeight,
        &rantzsoft_spatial2d::components::Position2D,
    )> = app
        .world_mut()
        .query::<(
            &CellWidth,
            &CellHeight,
            &rantzsoft_spatial2d::components::Position2D,
        )>()
        .iter(app.world())
        .collect();
    assert_eq!(cells.len(), 1, "should spawn exactly 1 cell");

    let (w, h, pos) = cells[0];
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "single cell should be centered at x=0.0, got {}",
        pos.0.x
    );
    assert!(
        (w.value - 126.0).abs() < f32::EPSILON,
        "1x1 grid CellWidth should be 126.0, got {}",
        w.value
    );
    assert!(
        (h.value - 43.0).abs() < f32::EPSILON,
        "1x1 grid CellHeight should be 43.0, got {}",
        h.value
    );
}
