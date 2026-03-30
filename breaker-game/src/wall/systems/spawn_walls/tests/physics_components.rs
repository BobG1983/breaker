//! Tests for physics components on spawned walls — `Aabb2D` and `CollisionLayers`.

use bevy::prelude::*;

use super::helpers::test_app;
use crate::{shared::PlayfieldConfig, wall::components::Wall};

// --- Aabb2D + CollisionLayers tests ---

#[test]
fn side_walls_have_aabb2d_matching_wall_size() {
    // Given: PlayfieldConfig default (width=800, height=600, wall_thickness=180.0)
    //        wall_half_thickness=90.0, half_height=300.0
    // When: spawn_walls runs
    // Then: left and right walls have Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(90.0, 300.0) }
    use rantzsoft_physics2d::aabb::Aabb2D;
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = test_app();
    app.update();
    let playfield = PlayfieldConfig::default();
    let wall_ht = playfield.wall_half_thickness(); // 90.0
    let half_height = playfield.height / 2.0; // 300.0

    let wall_data: Vec<(Vec2, &Aabb2D)> = app
        .world_mut()
        .query_filtered::<(&Position2D, &Aabb2D), With<Wall>>()
        .iter(app.world())
        .map(|(pos, aabb)| (pos.0, aabb))
        .collect();

    assert_eq!(wall_data.len(), 3, "should have 3 walls with Aabb2D");

    // Left wall: x < playfield.left()
    let left = wall_data
        .iter()
        .find(|(pos, _)| pos.x < playfield.left())
        .expect("should have left wall with Aabb2D");
    assert_eq!(
        left.1.center,
        Vec2::ZERO,
        "left wall Aabb2D center should be ZERO (local space)"
    );
    assert!(
        (left.1.half_extents.x - wall_ht).abs() < f32::EPSILON
            && (left.1.half_extents.y - half_height).abs() < f32::EPSILON,
        "left wall Aabb2D half_extents should be ({wall_ht}, {half_height}), got ({}, {})",
        left.1.half_extents.x,
        left.1.half_extents.y,
    );

    // Right wall: x > playfield.right() — same dimensions as left
    let right = wall_data
        .iter()
        .find(|(pos, _)| pos.x > playfield.right())
        .expect("should have right wall with Aabb2D");
    assert_eq!(right.1.center, Vec2::ZERO);
    assert!(
        (right.1.half_extents.x - wall_ht).abs() < f32::EPSILON
            && (right.1.half_extents.y - half_height).abs() < f32::EPSILON,
        "right wall Aabb2D half_extents should be ({wall_ht}, {half_height}), got ({}, {})",
        right.1.half_extents.x,
        right.1.half_extents.y,
    );
}

#[test]
fn ceiling_wall_has_aabb2d_with_different_dimensions_from_side_walls() {
    // Given: PlayfieldConfig default, half_width=400.0, wall_half_thickness=90.0
    // When: spawn_walls runs
    // Then: ceiling entity has Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(400.0, 90.0) }
    use rantzsoft_physics2d::aabb::Aabb2D;
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = test_app();
    app.update();
    let playfield = PlayfieldConfig::default();
    let wall_ht = playfield.wall_half_thickness(); // 90.0
    let half_width = playfield.width / 2.0; // 400.0

    let wall_data: Vec<(Vec2, &Aabb2D)> = app
        .world_mut()
        .query_filtered::<(&Position2D, &Aabb2D), With<Wall>>()
        .iter(app.world())
        .map(|(pos, aabb)| (pos.0, aabb))
        .collect();

    // Ceiling: y > playfield.top()
    let ceiling = wall_data
        .iter()
        .find(|(pos, _)| pos.y > playfield.top())
        .expect("should have ceiling wall with Aabb2D");
    assert_eq!(
        ceiling.1.center,
        Vec2::ZERO,
        "ceiling Aabb2D center should be ZERO (local space)"
    );
    assert!(
        (ceiling.1.half_extents.x - half_width).abs() < f32::EPSILON
            && (ceiling.1.half_extents.y - wall_ht).abs() < f32::EPSILON,
        "ceiling Aabb2D half_extents should be ({half_width}, {wall_ht}), got ({}, {})",
        ceiling.1.half_extents.x,
        ceiling.1.half_extents.y,
    );

    // Edge case: ceiling x is half_width, NOT wall_half_thickness
    assert!(
        (ceiling.1.half_extents.x - half_width).abs() < f32::EPSILON,
        "ceiling Aabb2D.half_extents.x should be half_width ({half_width}), not wall_ht ({wall_ht})"
    );
}

#[test]
fn all_walls_have_collision_layers_wall_membership_bolt_mask() {
    // Given: spawn_walls runs
    // Then: all 3 walls have CollisionLayers { membership: WALL_LAYER (0x04), mask: BOLT_LAYER (0x01) }
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::{BOLT_LAYER, WALL_LAYER};

    let mut app = test_app();
    app.update();

    let layers_list: Vec<&CollisionLayers> = app
        .world_mut()
        .query_filtered::<&CollisionLayers, With<Wall>>()
        .iter(app.world())
        .collect();

    assert_eq!(
        layers_list.len(),
        3,
        "all 3 walls should have CollisionLayers"
    );
    for (i, layers) in layers_list.iter().enumerate() {
        assert_eq!(
            layers.membership, WALL_LAYER,
            "wall {i} membership should be WALL_LAYER (0x{:02X}), got 0x{:02X}",
            WALL_LAYER, layers.membership,
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "wall {i} mask should be BOLT_LAYER (0x{:02X}), got 0x{:02X}",
            BOLT_LAYER, layers.mask,
        );
    }
}
