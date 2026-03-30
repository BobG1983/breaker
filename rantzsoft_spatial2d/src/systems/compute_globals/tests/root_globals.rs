use bevy::prelude::*;

use super::{super::system::*, helpers::tick};
use crate::components::{
    GlobalPosition2D, GlobalRotation2D, GlobalScale2D, Position2D, Rotation2D, Scale2D, Spatial2D,
};

// -- Behavior 13: Root entity `GlobalPosition2D` = `Position2D` --

#[test]
fn root_entity_global_position_equals_local() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let entity = app
        .world_mut()
        .spawn((Spatial2D, Position2D(Vec2::new(100.0, 200.0))))
        .id();

    tick(&mut app);

    let global_pos = app.world().get::<GlobalPosition2D>(entity).unwrap();
    assert_eq!(
        global_pos.0,
        Vec2::new(100.0, 200.0),
        "root GlobalPosition2D should equal Position2D"
    );
}

// -- Behavior 14: Root entity `GlobalRotation2D` = `Rotation2D` --

#[test]
fn root_entity_global_rotation_equals_local() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let entity = app
        .world_mut()
        .spawn((Spatial2D, Rotation2D::from_degrees(45.0)))
        .id();

    tick(&mut app);

    let global_rot = app.world().get::<GlobalRotation2D>(entity).unwrap();
    assert!(
        (global_rot.0.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
        "root GlobalRotation2D should equal Rotation2D (~PI/4), got {}",
        global_rot.0.as_radians()
    );
}

// -- Behavior 15: Root entity `GlobalScale2D` = `Scale2D` --

#[test]
fn root_entity_global_scale_equals_local() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let entity = app
        .world_mut()
        .spawn((Spatial2D, Scale2D { x: 2.0, y: 3.0 }))
        .id();

    tick(&mut app);

    let global_scale = app.world().get::<GlobalScale2D>(entity).unwrap();
    assert!(
        (global_scale.x - 2.0).abs() < f32::EPSILON,
        "root GlobalScale2D.x should be 2.0, got {}",
        global_scale.x
    );
    assert!(
        (global_scale.y - 3.0).abs() < f32::EPSILON,
        "root GlobalScale2D.y should be 3.0, got {}",
        global_scale.y
    );
}
