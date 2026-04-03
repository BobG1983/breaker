use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::helpers::default_playfield;
use crate::{shared::PlayfieldConfig, wall::components::Wall};

// ── Behavior 40: Left wall position scales with custom playfield width ──

#[test]
fn left_wall_custom_width_1000() {
    let pf = PlayfieldConfig {
        width: 1000.0,
        height: 600.0,
        ..default_playfield()
    };
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-590.0)).abs() < f32::EPSILON,
        "Left with width=1000: x = left - ht = -500 - 90 = -590.0, got {}",
        pos.0.x
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 90.0).abs() < f32::EPSILON,
        "Scale2D.x should be 90.0"
    );
    assert!(
        (scale.y - 300.0).abs() < f32::EPSILON,
        "Scale2D.y should be 300.0"
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 90.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.x should be 90.0"
    );
    assert!(
        (aabb.half_extents.y - 300.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.y should be 300.0"
    );
}

#[test]
fn left_wall_zero_width() {
    let pf = PlayfieldConfig {
        width: 0.0,
        height: 600.0,
        ..default_playfield()
    };
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-90.0)).abs() < f32::EPSILON,
        "Left with width=0: x = 0 - 90 = -90.0, got {}",
        pos.0.x
    );
}

// ── Behavior 41: Ceiling wall position scales with custom playfield height ──

#[test]
fn ceiling_wall_custom_height_1080() {
    let pf = PlayfieldConfig {
        width: 800.0,
        height: 1080.0,
        ..default_playfield()
    };
    let mut world = World::new();

    let bundle = Wall::builder().ceiling(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(pos.0.x.abs() < f32::EPSILON, "Ceiling x should be 0.0");
    assert!(
        (pos.0.y - 630.0).abs() < f32::EPSILON,
        "Ceiling with height=1080: y = top + ht = 540 + 90 = 630.0, got {}",
        pos.0.y
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 400.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.x should be 400.0"
    );
    assert!(
        (scale.y - 90.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.y should be 90.0"
    );
}

#[test]
fn ceiling_wall_zero_height() {
    let pf = PlayfieldConfig {
        width: 800.0,
        height: 0.0,
        ..default_playfield()
    };
    let mut world = World::new();

    let bundle = Wall::builder().ceiling(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - 90.0).abs() < f32::EPSILON,
        "Ceiling with height=0: y = 0 + 90 = 90.0, got {}",
        pos.0.y
    );
}

// ── Behavior 42: Floor wall position uses bottom edge directly ──

#[test]
fn floor_wall_custom_height_1080() {
    let pf = PlayfieldConfig {
        width: 800.0,
        height: 1080.0,
        ..default_playfield()
    };
    let mut world = World::new();

    let bundle = Wall::builder().floor(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-540.0)).abs() < f32::EPSILON,
        "Floor with height=1080: y = bottom = -540.0 (NOT bottom - ht), got {}",
        pos.0.y
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 400.0).abs() < f32::EPSILON,
        "Floor Scale2D.x should be 400.0 (half_width)"
    );
    assert!(
        (scale.y - 90.0).abs() < f32::EPSILON,
        "Floor Scale2D.y should be 90.0"
    );
}

#[test]
fn floor_wall_default_playfield_at_bottom() {
    let pf = default_playfield();
    let mut world = World::new();

    let bundle = Wall::builder().floor(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Floor with default playfield: y = -300.0, got {}",
        pos.0.y
    );
}
