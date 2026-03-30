//! Tests for `Velocity2D` -> `PreviousVelocity` snapshotting in `save_previous`.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::components::*;

#[test]
fn snapshots_velocity_to_previous_velocity() {
    use crate::components::{PreviousVelocity, Velocity2D};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    let entity = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            Velocity2D(Vec2::new(0.0, 400.0)),
            PreviousVelocity(Vec2::ZERO),
            Position2D::default(),
            PreviousPosition::default(),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousVelocity>(entity).unwrap();
    assert_eq!(
        prev.0,
        Vec2::new(0.0, 400.0),
        "PreviousVelocity should snapshot Velocity2D (0, 400)"
    );
}

#[test]
fn skips_velocity_snapshot_without_velocity_component() {
    use crate::components::PreviousVelocity;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    // Entity has PreviousVelocity but NOT Velocity2D.
    let entity = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            PreviousVelocity(Vec2::new(99.0, 99.0)),
            Position2D::default(),
            PreviousPosition::default(),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousVelocity>(entity).unwrap();
    assert_eq!(
        prev.0,
        Vec2::new(99.0, 99.0),
        "PreviousVelocity should be unchanged when entity lacks Velocity2D"
    );
}

#[test]
fn skips_velocity_snapshot_without_interpolate_marker() {
    use crate::components::{PreviousVelocity, Velocity2D};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    // No InterpolateTransform2D marker.
    let entity = app
        .world_mut()
        .spawn((
            Velocity2D(Vec2::new(0.0, 400.0)),
            PreviousVelocity(Vec2::ZERO),
        ))
        .id();

    tick(&mut app);

    let prev = app.world().get::<PreviousVelocity>(entity).unwrap();
    assert_eq!(
        prev.0,
        Vec2::ZERO,
        "PreviousVelocity should be unchanged without InterpolateTransform2D"
    );
}
