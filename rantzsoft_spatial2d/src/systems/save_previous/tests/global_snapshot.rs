//! Tests for `Global*` -> `Previous*` snapshotting in `save_previous`.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::components::*;

#[test]
fn snapshots_global_position_to_previous_position() {
    use crate::components::GlobalPosition2D;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    let entity = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            GlobalPosition2D(Vec2::new(10.0, 20.0)),
            PreviousPosition(Vec2::ZERO),
            // Local position differs from global -- save_previous should use Global*.
            Position2D(Vec2::new(999.0, 999.0)),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousPosition>(entity).unwrap();
    assert_eq!(
        prev.0,
        Vec2::new(10.0, 20.0),
        "PreviousPosition should snapshot GlobalPosition2D (10, 20), not local Position2D"
    );
}

#[test]
fn snapshots_global_rotation_to_previous_rotation() {
    use crate::components::GlobalRotation2D;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    let entity = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            GlobalRotation2D(Rot2::degrees(45.0)),
            PreviousRotation::default(),
            // Local rotation differs from global.
            Rotation2D::from_degrees(999.0),
            Position2D::default(),
            PreviousPosition::default(),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousRotation>(entity).unwrap();
    assert!(
        (prev.0.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
        "PreviousRotation should snapshot GlobalRotation2D (~45 degrees), got {} radians",
        prev.0.as_radians()
    );
}

#[test]
fn snapshots_global_scale_to_previous_scale() {
    use crate::components::GlobalScale2D;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    let entity = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            GlobalScale2D { x: 3.0, y: 4.0 },
            PreviousScale { x: 1.0, y: 1.0 },
            // Local scale differs from global.
            Scale2D { x: 999.0, y: 999.0 },
            Position2D::default(),
            PreviousPosition::default(),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev.x - 3.0).abs() < f32::EPSILON,
        "PreviousScale.x should snapshot GlobalScale2D.x (3.0), got {}",
        prev.x
    );
    assert!(
        (prev.y - 4.0).abs() < f32::EPSILON,
        "PreviousScale.y should snapshot GlobalScale2D.y (4.0), got {}",
        prev.y
    );
}
