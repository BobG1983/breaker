//! Tests for spatial components on spawned walls — `Position2D`, `GameDrawLayer`,
//! `Spatial2D`, and `Scale2D`.

use bevy::prelude::*;

use super::helpers::test_app;
use crate::{shared::PlayfieldConfig, walls::components::Wall};

// --- Position2D migration tests ---

#[test]
fn spawned_walls_have_position2d_at_correct_positions() {
    // Given: PlayfieldConfig default (width=800, height=600, wall_thickness=180.0)
    // When: spawn_walls runs
    // Then: Three walls with Position2D at computed positions:
    //   Left:    x = playfield.left() - wall_half_thickness = -400.0 - 90.0 = -490.0, y = 0.0
    //   Right:   x = playfield.right() + wall_half_thickness = 400.0 + 90.0 = 490.0, y = 0.0
    //   Ceiling: x = 0.0, y = playfield.top() + wall_half_thickness = 300.0 + 90.0 = 390.0
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = test_app();
    app.update();
    let playfield = PlayfieldConfig::default();
    let wall_ht = playfield.wall_half_thickness(); // 90.0

    let positions: Vec<Vec2> = app
        .world_mut()
        .query_filtered::<&Position2D, With<Wall>>()
        .iter(app.world())
        .map(|p| p.0)
        .collect();

    assert_eq!(positions.len(), 3, "should have 3 walls with Position2D");

    // Left wall
    let left = positions
        .iter()
        .find(|p| p.x < playfield.left())
        .expect("should have left wall Position2D");
    let expected_left_x = playfield.left() - wall_ht; // -490.0
    assert!(
        (left.x - expected_left_x).abs() < f32::EPSILON,
        "left wall x should be {expected_left_x}, got {}",
        left.x
    );
    assert!(
        left.y.abs() < f32::EPSILON,
        "left wall y should be 0.0, got {}",
        left.y
    );

    // Right wall
    let right = positions
        .iter()
        .find(|p| p.x > playfield.right())
        .expect("should have right wall Position2D");
    let expected_right_x = playfield.right() + wall_ht; // 490.0
    assert!(
        (right.x - expected_right_x).abs() < f32::EPSILON,
        "right wall x should be {expected_right_x}, got {}",
        right.x
    );
    assert!(
        right.y.abs() < f32::EPSILON,
        "right wall y should be 0.0, got {}",
        right.y
    );

    // Ceiling
    let ceiling = positions
        .iter()
        .find(|p| p.y > playfield.top())
        .expect("should have ceiling wall Position2D");
    let expected_ceiling_y = playfield.top() + wall_ht; // 390.0
    assert!(
        ceiling.x.abs() < f32::EPSILON,
        "ceiling x should be 0.0, got {}",
        ceiling.x
    );
    assert!(
        (ceiling.y - expected_ceiling_y).abs() < f32::EPSILON,
        "ceiling y should be {expected_ceiling_y}, got {}",
        ceiling.y
    );
}

#[test]
fn spawned_walls_have_game_draw_layer_wall() {
    // Given: PlayfieldConfig default
    // When: spawn_walls runs
    // Then: Each Wall has GameDrawLayer::Wall (z=0.0)
    use rantzsoft_spatial2d::draw_layer::DrawLayer;

    use crate::shared::GameDrawLayer;

    let mut app = test_app();
    app.update();

    let layers: Vec<&GameDrawLayer> = app
        .world_mut()
        .query_filtered::<&GameDrawLayer, With<Wall>>()
        .iter(app.world())
        .collect();

    assert_eq!(layers.len(), 3, "all 3 walls should have GameDrawLayer");
    for layer in &layers {
        assert!(
            layer.z().abs() < f32::EPSILON,
            "GameDrawLayer::Wall.z() should be 0.0, got {}",
            layer.z()
        );
    }
}

#[test]
fn spawned_walls_have_spatial2d_and_spatial_markers() {
    // Given: PlayfieldConfig default
    // When: spawn_walls runs
    // Then: Each Wall has Spatial2D and Spatial (via builder's Spatial::builder()).
    use rantzsoft_spatial2d::components::{Spatial, Spatial2D};

    let mut app = test_app();
    app.update();

    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Wall>>()
        .iter(app.world())
        .collect();

    assert_eq!(entities.len(), 3, "should have 3 wall entities");
    for entity in &entities {
        assert!(
            app.world().get::<Spatial2D>(*entity).is_some(),
            "wall should have Spatial2D marker"
        );
        assert!(
            app.world().get::<Spatial>(*entity).is_some(),
            "wall should have Spatial marker (from builder)"
        );
    }
}

#[test]
fn spawned_walls_have_scale2d_matching_wall_dimensions() {
    // Given: PlayfieldConfig default (width=800, height=600, wall_thickness=180.0)
    // When: spawn_walls runs
    // Then: Left/right walls have Scale2D { x: wall_half_thickness, y: half_height }
    //       Ceiling has Scale2D { x: half_width, y: wall_half_thickness }
    //       wall_half_thickness=90.0, half_width=400.0, half_height=300.0
    use rantzsoft_spatial2d::components::Scale2D;

    let mut app = test_app();
    app.update();
    let playfield = PlayfieldConfig::default();
    let wall_ht = playfield.wall_half_thickness(); // 90.0
    let half_width = playfield.width / 2.0; // 400.0
    let half_height = playfield.height / 2.0; // 300.0

    // Correlate Scale2D with position to identify which wall is which.
    let wall_data: Vec<&Scale2D> = app
        .world_mut()
        .query_filtered::<&Scale2D, With<Wall>>()
        .iter(app.world())
        .collect();

    assert_eq!(wall_data.len(), 3, "all 3 walls should have Scale2D");

    // Left/right walls: Scale2D { x: wall_ht, y: half_height }
    let side_walls: Vec<_> = wall_data
        .iter()
        .filter(|scale| {
            (scale.x - wall_ht).abs() < f32::EPSILON && (scale.y - half_height).abs() < f32::EPSILON
        })
        .collect();
    assert_eq!(side_walls.len(), 2, "should have 2 side walls");
    for scale in &side_walls {
        assert!(
            (scale.x - wall_ht).abs() < f32::EPSILON,
            "side wall Scale2D.x should be {wall_ht}, got {}",
            scale.x
        );
        assert!(
            (scale.y - half_height).abs() < f32::EPSILON,
            "side wall Scale2D.y should be {half_height}, got {}",
            scale.y
        );
    }

    // Ceiling: Scale2D { x: half_width, y: wall_ht }
    let ceiling_walls: Vec<_> = wall_data
        .iter()
        .filter(|scale| {
            (scale.x - half_width).abs() < f32::EPSILON && (scale.y - wall_ht).abs() < f32::EPSILON
        })
        .collect();
    assert_eq!(ceiling_walls.len(), 1, "should have 1 ceiling wall");
    let ceiling_scale = ceiling_walls[0];
    assert!(
        (ceiling_scale.x - half_width).abs() < f32::EPSILON,
        "ceiling Scale2D.x should be {half_width}, got {}",
        ceiling_scale.x
    );
    assert!(
        (ceiling_scale.y - wall_ht).abs() < f32::EPSILON,
        "ceiling Scale2D.y should be {wall_ht}, got {}",
        ceiling_scale.y
    );
}
