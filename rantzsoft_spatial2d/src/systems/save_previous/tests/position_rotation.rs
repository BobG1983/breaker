//! Tests for position and rotation snapshotting in `save_previous`.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::components::*;

#[test]
fn copies_position_to_previous_position() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    app.world_mut().spawn((
        InterpolateTransform2D,
        Position2D(Vec2::new(10.0, 20.0)),
        PreviousPosition(Vec2::ZERO),
        Rotation2D::default(),
        PreviousRotation::default(),
    ));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousPosition>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert_eq!(
        prev.0,
        Vec2::new(10.0, 20.0),
        "PreviousPosition should match current Position2D"
    );
}

#[test]
fn copies_rotation_to_previous_rotation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    app.world_mut().spawn((
        InterpolateTransform2D,
        Position2D::default(),
        PreviousPosition::default(),
        Rotation2D::from_degrees(45.0),
        PreviousRotation::default(),
    ));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousRotation>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert!(
        (prev.0.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
        "PreviousRotation should be ~45 degrees, got {} radians",
        prev.0.as_radians()
    );
}

#[test]
fn skips_entity_without_interpolate_marker_position() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    // No InterpolateTransform2D marker
    app.world_mut().spawn((
        Position2D(Vec2::new(99.0, 99.0)),
        PreviousPosition(Vec2::ZERO),
    ));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousPosition>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert_eq!(
        prev.0,
        Vec2::ZERO,
        "PreviousPosition should be unchanged without InterpolateTransform2D"
    );
}

#[test]
fn skips_entity_without_interpolate_marker_rotation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    // No InterpolateTransform2D marker
    app.world_mut()
        .spawn((Rotation2D::from_degrees(90.0), PreviousRotation::default()));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousRotation>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert!(
        prev.0.as_radians().abs() < 1e-6,
        "PreviousRotation should be unchanged without InterpolateTransform2D"
    );
}
