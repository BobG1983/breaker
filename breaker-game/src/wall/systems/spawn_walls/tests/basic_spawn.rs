//! Basic spawn tests — wall count, cleanup marker, message dispatch,
//! and position matching.

use bevy::prelude::*;

use super::helpers::test_app;
use crate::{
    shared::{CleanupOnNodeExit, PlayfieldConfig},
    wall::{
        components::Wall, definition::WallDefinition, messages::WallsSpawned,
        registry::WallRegistry,
    },
};

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

#[test]
fn spawn_walls_uses_definition_half_thickness_from_registry() {
    use rantzsoft_spatial2d::components::{Position2D, Scale2D};

    // Given: WallRegistry with a custom half_thickness of 45.0
    let mut registry = WallRegistry::default();
    registry.insert(
        "Wall".to_string(),
        WallDefinition {
            name: "Wall".to_string(),
            half_thickness: 45.0,
            ..WallDefinition::default()
        },
    );

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<WallsSpawned>()
        .init_resource::<PlayfieldConfig>()
        .insert_resource(registry)
        .add_systems(Update, super::super::spawn_walls);
    app.update();

    // Verify: Left wall Position2D.x == -445.0, Scale2D.x == 45.0
    // playfield.left() = -400.0, so left wall x = -400.0 - 45.0 = -445.0
    let playfield = PlayfieldConfig::default();

    let wall_data: Vec<(Vec2, f32, f32)> = app
        .world_mut()
        .query_filtered::<(&Position2D, &Scale2D), With<Wall>>()
        .iter(app.world())
        .map(|(pos, scale)| (pos.0, scale.x, scale.y))
        .collect();

    assert_eq!(wall_data.len(), 3, "should spawn 3 walls");

    // Left wall: x < playfield.left()
    let left = wall_data
        .iter()
        .find(|(pos, ..)| pos.x < playfield.left())
        .expect("should have left wall");
    assert!(
        (left.0.x - (-445.0)).abs() < f32::EPSILON,
        "left wall Position2D.x should be -445.0, got {}",
        left.0.x
    );
    assert!(
        (left.1 - 45.0).abs() < f32::EPSILON,
        "left wall Scale2D.x should be 45.0, got {}",
        left.1
    );
}
