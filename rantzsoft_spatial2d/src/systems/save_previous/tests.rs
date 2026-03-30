use bevy::prelude::*;

use super::system::*;
use crate::components::*;

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

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

// ── Global* -> Previous* snapshotting ───────────────────────

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
            // Local position differs from global — save_previous should use Global*.
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

// ── Velocity2D -> PreviousVelocity snapshotting ─────────────

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
