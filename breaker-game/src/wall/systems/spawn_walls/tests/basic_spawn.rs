//! Basic spawn tests — wall count, `WallSize`, cleanup marker, message dispatch,
//! and position matching.

use bevy::prelude::*;

use super::helpers::test_app;
use crate::{
    shared::{CleanupOnNodeExit, PlayfieldConfig},
    wall::{
        components::{Wall, WallSize},
        messages::WallsSpawned,
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
