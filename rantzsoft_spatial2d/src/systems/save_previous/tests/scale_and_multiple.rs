//! Tests for scale snapshotting and multiple-entity updates in `save_previous`.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::components::*;

#[test]
fn multiple_entities_all_updated() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    let e1 = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            Position2D(Vec2::new(1.0, 2.0)),
            PreviousPosition(Vec2::ZERO),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    let e2 = app
        .world_mut()
        .spawn((
            InterpolateTransform2D,
            Position2D(Vec2::new(30.0, 40.0)),
            PreviousPosition(Vec2::ZERO),
            Rotation2D::default(),
            PreviousRotation::default(),
        ))
        .id();

    tick(&mut app);

    let prev1 = app.world().get::<PreviousPosition>(e1).expect("e1 exists");
    let prev2 = app.world().get::<PreviousPosition>(e2).expect("e2 exists");

    assert_eq!(
        prev1.0,
        Vec2::new(1.0, 2.0),
        "first entity PreviousPosition should be updated"
    );
    assert_eq!(
        prev2.0,
        Vec2::new(30.0, 40.0),
        "second entity PreviousPosition should be updated"
    );
}

// ── PreviousScale ─────────────────────────────────────────

#[test]
fn copies_scale_to_previous_scale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    app.world_mut().spawn((
        InterpolateTransform2D,
        Scale2D { x: 3.0, y: 4.0 },
        PreviousScale { x: 1.0, y: 1.0 },
        Position2D::default(),
        PreviousPosition::default(),
        Rotation2D::default(),
        PreviousRotation::default(),
    ));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousScale>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert!(
        (prev.x - 3.0).abs() < f32::EPSILON,
        "PreviousScale.x should match current Scale2D.x (3.0), got {}",
        prev.x
    );
    assert!(
        (prev.y - 4.0).abs() < f32::EPSILON,
        "PreviousScale.y should match current Scale2D.y (4.0), got {}",
        prev.y
    );
}

#[test]
fn skips_scale_without_interpolate_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, save_previous);

    // No InterpolateTransform2D marker.
    app.world_mut()
        .spawn((Scale2D { x: 5.0, y: 6.0 }, PreviousScale { x: 1.0, y: 1.0 }));

    tick(&mut app);

    let prev = app
        .world_mut()
        .query::<&PreviousScale>()
        .iter(app.world())
        .next()
        .expect("entity should exist");

    assert!(
        (prev.x - 1.0).abs() < f32::EPSILON,
        "PreviousScale.x should remain 1.0 without InterpolateTransform2D, got {}",
        prev.x
    );
    assert!(
        (prev.y - 1.0).abs() < f32::EPSILON,
        "PreviousScale.y should remain 1.0 without InterpolateTransform2D, got {}",
        prev.y
    );
}
