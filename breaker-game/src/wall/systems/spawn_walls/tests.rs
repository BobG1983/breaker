use bevy::prelude::*;

use super::*;
use crate::{
    shared::{CleanupOnNodeExit, PlayfieldConfig},
    wall::{
        components::{Wall, WallSize},
        messages::WallsSpawned,
    },
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WallsSpawned>()
        .init_resource::<PlayfieldConfig>()
        .add_systems(Update, spawn_walls);
    app
}

#[test]
fn spawns_three_walls() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Wall>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 3, "should spawn left, right, and ceiling walls");
}

#[test]
fn walls_have_wall_size() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query::<&WallSize>()
        .iter(app.world())
        .count();
    assert_eq!(count, 3);
}

#[test]
fn walls_have_cleanup_marker() {
    let mut app = test_app();
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, (With<Wall>, With<CleanupOnNodeExit>)>()
        .iter(app.world())
        .count();
    assert_eq!(count, 3, "all walls should have CleanupOnNodeExit");
}

#[test]
fn spawn_walls_sends_walls_spawned_message() {
    let mut app = test_app();
    app.update();

    let messages = app.world().resource::<Messages<WallsSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_walls must send WallsSpawned message"
    );
}

#[test]
fn wall_positions_match_playfield() {
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = test_app();
    app.update();
    let playfield = PlayfieldConfig::default();

    let walls: Vec<_> = app
        .world_mut()
        .query::<&Position2D>()
        .iter(app.world())
        .map(|pos| pos.0)
        .collect();

    // Left wall: x < playfield left
    let left = walls
        .iter()
        .find(|pos| pos.x < playfield.left())
        .expect("should have left wall");
    assert!((left.y).abs() < f32::EPSILON, "left wall centered at y=0");

    // Right wall: x > playfield right
    let right = walls
        .iter()
        .find(|pos| pos.x > playfield.right())
        .expect("should have right wall");
    assert!((right.y).abs() < f32::EPSILON, "right wall centered at y=0");

    // Ceiling: y > playfield top
    let ceiling = walls
        .iter()
        .find(|pos| pos.y > playfield.top())
        .expect("should have ceiling wall");
    assert!((ceiling.x).abs() < f32::EPSILON, "ceiling centered at x=0");
}

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
fn spawned_walls_have_spatial2d_but_not_interpolate_transform2d() {
    // Given: PlayfieldConfig default
    // When: spawn_walls runs
    // Then: Each Wall has Spatial2D. Does NOT have InterpolateTransform2D.
    use rantzsoft_spatial2d::components::{InterpolateTransform2D, Spatial2D};

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
            app.world().get::<InterpolateTransform2D>(*entity).is_none(),
            "wall should NOT have InterpolateTransform2D (static entity)"
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

    // We need to correlate Scale2D with position to identify which wall is which.
    // Use Scale2D to identify walls since WallSize fields are commented out.
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
