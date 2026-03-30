//! Tests for `SecondWind` `fire()` wall spawning and component setup.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::super::system::*;
use crate::{
    shared::{BOLT_LAYER, PlayfieldConfig, WALL_LAYER},
    wall::components::Wall,
};

#[test]
fn fire_spawns_second_wind_wall() {
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    fire(entity, "", &mut world);

    let walls: Vec<Entity> = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .collect();
    assert_eq!(
        walls.len(),
        1,
        "fire should spawn one SecondWindWall entity"
    );
}

#[test]
fn reverse_despawns_all_second_wind_walls() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    world.spawn((
        SecondWindWall,
        Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
    ));
    world.spawn((
        SecondWindWall,
        Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
    ));

    reverse(entity, "", &mut world);

    let remaining: Vec<Entity> = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .collect();
    assert!(
        remaining.is_empty(),
        "reverse should despawn all SecondWindWall entities"
    );
}

// =========================================================================
// Wave 4B: SecondWind Collision Components
// =========================================================================

#[test]
fn fire_spawns_wall_with_wall_marker() {
    // Behavior 13: SecondWind fire spawns wall with Wall marker component.
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    fire(entity, "", &mut world);

    let count = world
        .query_filtered::<Entity, (With<SecondWindWall>, With<Wall>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 1,
        "SecondWind wall should have Wall marker component"
    );
}

#[test]
fn fire_spawns_wall_with_position2d_at_playfield_bottom() {
    // Behavior 14: SecondWind wall position at playfield bottom.
    // Given: PlayfieldConfig::default() (width=800, height=600, bottom()=-300.0).
    // When: fire spawns the wall
    // Then: Position2D is Vec2::new(0.0, -300.0).
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    fire(entity, "", &mut world);

    let positions: Vec<Vec2> = world
        .query_filtered::<&Position2D, With<SecondWindWall>>()
        .iter(&world)
        .map(|p| p.0)
        .collect();
    assert_eq!(positions.len(), 1, "should spawn one wall with Position2D");
    let pos = positions[0];
    assert!(
        (pos.x - 0.0).abs() < f32::EPSILON,
        "SecondWind wall x should be 0.0, got {:.1}",
        pos.x
    );
    assert!(
        (pos.y - (-300.0)).abs() < f32::EPSILON,
        "SecondWind wall y should be -300.0 (playfield bottom), got {:.1}",
        pos.y
    );
}

#[test]
fn fire_spawns_wall_with_collision_layers_matching_other_walls() {
    // Behavior 15: SecondWind wall has CollisionLayers matching other walls.
    // membership: WALL_LAYER (0x04), mask: BOLT_LAYER (0x01).
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    fire(entity, "", &mut world);

    let layers: Vec<&CollisionLayers> = world
        .query_filtered::<&CollisionLayers, With<SecondWindWall>>()
        .iter(&world)
        .collect();
    assert_eq!(
        layers.len(),
        1,
        "wall should have CollisionLayers component"
    );
    assert_eq!(
        layers[0].membership, WALL_LAYER,
        "wall membership should be WALL_LAYER (0x{WALL_LAYER:02X}), got 0x{:02X}",
        layers[0].membership
    );
    assert_eq!(
        layers[0].mask, BOLT_LAYER,
        "wall mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
        layers[0].mask
    );
}

#[test]
fn fire_spawns_wall_with_aabb2d_spanning_playfield_width() {
    // Behavior 16: SecondWind wall has Aabb2D spanning playfield width.
    // Given: PlayfieldConfig::default() (width=800, half_width=400).
    // Then: Aabb2D center=Vec2::ZERO, half_extents=(400.0, wall_half_thickness).
    let mut world = World::new();
    let playfield = PlayfieldConfig::default();
    world.insert_resource(playfield.clone());
    let entity = world.spawn_empty().id();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    fire(entity, "", &mut world);

    let aabbs: Vec<&Aabb2D> = world
        .query_filtered::<&Aabb2D, With<SecondWindWall>>()
        .iter(&world)
        .collect();
    assert_eq!(aabbs.len(), 1, "wall should have Aabb2D component");
    let aabb = aabbs[0];
    assert_eq!(
        aabb.center,
        Vec2::ZERO,
        "Aabb2D center should be Vec2::ZERO, got {:?}",
        aabb.center
    );
    assert!(
        (aabb.half_extents.x - half_width).abs() < f32::EPSILON,
        "Aabb2D half_extents.x should be {half_width:.1}, got {:.1}",
        aabb.half_extents.x
    );
    assert!(
        (aabb.half_extents.y - wall_ht).abs() < f32::EPSILON,
        "Aabb2D half_extents.y should be {wall_ht:.1}, got {:.1}",
        aabb.half_extents.y
    );
}

#[test]
fn fire_spawns_wall_with_scale2d_matching_playfield() {
    // Behavior 13: Scale2D matches playfield half-width and wall half-thickness.
    let mut world = World::new();
    let playfield = PlayfieldConfig::default();
    world.insert_resource(playfield.clone());
    let entity = world.spawn_empty().id();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    fire(entity, "", &mut world);

    let scales: Vec<&Scale2D> = world
        .query_filtered::<&Scale2D, With<SecondWindWall>>()
        .iter(&world)
        .collect();
    assert_eq!(scales.len(), 1, "wall should have Scale2D component");
    let scale = scales[0];
    assert!(
        (scale.x - half_width).abs() < f32::EPSILON,
        "Scale2D.x should be {half_width:.1}, got {:.1}",
        scale.x
    );
    assert!(
        (scale.y - wall_ht).abs() < f32::EPSILON,
        "Scale2D.y should be {wall_ht:.1}, got {:.1}",
        scale.y
    );
}
