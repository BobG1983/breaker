use bevy::prelude::*;

use super::{super::system::*, helpers::tick};
use crate::{
    components::{
        GlobalPosition2D, GlobalRotation2D, GlobalScale2D, Position2D, Rotation2D, Scale2D,
        Spatial2D,
    },
    propagation::{PositionPropagation, RotationPropagation, ScalePropagation},
};

// -- Behavior 16: Relative child `GlobalPosition2D` = parent + child --

#[test]
fn relative_child_global_position_adds_parent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let parent = app
        .world_mut()
        .spawn((Spatial2D, Position2D(Vec2::new(100.0, 0.0))))
        .id();

    let child = app
        .world_mut()
        .spawn((
            ChildOf(parent),
            Spatial2D,
            Position2D(Vec2::new(50.0, 0.0)),
            PositionPropagation::Relative,
        ))
        .id();

    tick(&mut app);

    let global_pos = app.world().get::<GlobalPosition2D>(child).unwrap();
    assert_eq!(
        global_pos.0,
        Vec2::new(150.0, 0.0),
        "relative child GlobalPosition2D should be parent(100) + child(50) = 150"
    );
}

// -- Behavior 17: Relative child `GlobalRotation2D` = parent * child --

#[test]
fn relative_child_global_rotation_combines_parent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let parent = app
        .world_mut()
        .spawn((Spatial2D, Rotation2D::from_degrees(90.0)))
        .id();

    let child = app
        .world_mut()
        .spawn((
            ChildOf(parent),
            Spatial2D,
            Rotation2D::from_degrees(45.0),
            RotationPropagation::Relative,
        ))
        .id();

    tick(&mut app);

    let global_rot = app.world().get::<GlobalRotation2D>(child).unwrap();
    let expected_radians = 135.0_f32.to_radians();
    assert!(
        (global_rot.0.as_radians() - expected_radians).abs() < 1e-4,
        "relative child GlobalRotation2D should be ~135 degrees, got {} radians",
        global_rot.0.as_radians()
    );
}

// -- Behavior 18: Relative child `GlobalScale2D` = parent * child --

#[test]
fn relative_child_global_scale_multiplies_parent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, compute_globals);

    let parent = app
        .world_mut()
        .spawn((Spatial2D, Scale2D { x: 2.0, y: 2.0 }))
        .id();

    let child = app
        .world_mut()
        .spawn((
            ChildOf(parent),
            Spatial2D,
            Scale2D { x: 3.0, y: 4.0 },
            ScalePropagation::Relative,
        ))
        .id();

    tick(&mut app);

    let global_scale = app.world().get::<GlobalScale2D>(child).unwrap();
    assert!(
        (global_scale.x - 6.0).abs() < f32::EPSILON,
        "relative child GlobalScale2D.x should be 2*3=6, got {}",
        global_scale.x
    );
    assert!(
        (global_scale.y - 8.0).abs() < f32::EPSILON,
        "relative child GlobalScale2D.y should be 2*4=8, got {}",
        global_scale.y
    );
}
