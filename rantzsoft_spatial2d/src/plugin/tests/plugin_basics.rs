use bevy::prelude::*;

use super::{
    super::definition::*,
    helpers::{TestDrawLayer, tick},
};
use crate::components::*;

// -- Behavior 24: Plugin builds without panic --

#[test]
fn plugin_builds_without_panic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
    app.update();
    app.update();
}

// -- Behavior 35: Plugin registers `compute_globals` in `AfterFixedMainLoop` --

#[test]
fn plugin_computes_globals_for_root_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

    let entity = app
        .world_mut()
        .spawn((Spatial2D, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    tick(&mut app);

    let global_pos = app.world().get::<GlobalPosition2D>(entity).unwrap();
    assert_eq!(
        global_pos.0,
        Vec2::new(10.0, 20.0),
        "Plugin should run compute_globals: root GlobalPosition2D should equal Position2D"
    );
}

// -- Behavior 36: Plugin registers `derive_transform` in `AfterFixedMainLoop` --

#[test]
fn plugin_derives_transform_from_globals() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

    let entity = app
        .world_mut()
        .spawn((
            Spatial2D,
            Position2D(Vec2::new(10.0, 20.0)),
            TestDrawLayer::A,
        ))
        .id();

    tick(&mut app);

    let tf = app.world().get::<Transform>(entity).unwrap();
    assert_eq!(
        tf.translation,
        Vec3::new(10.0, 20.0, 0.0),
        "Plugin should run derive_transform: Transform should match GlobalPosition2D"
    );
}

// -- Behavior 37: Plugin registers `apply_velocity` for entities with marker --

#[test]
fn plugin_registers_apply_velocity_with_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

    let entity = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            ApplyVelocity,
        ))
        .id();

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    // dt = 1/64 = 0.015625, displacement = 400 * 0.015625 = 6.25
    assert!(
        (pos.0.y - 6.25).abs() < 1e-3,
        "Plugin should register apply_velocity: y should be ~6.25, got {}",
        pos.0.y
    );
}

#[test]
fn plugin_apply_velocity_skips_without_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

    let entity = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert_eq!(
        pos.0,
        Vec2::ZERO,
        "Without ApplyVelocity marker, Position2D should be unchanged"
    );
}

// -- Behavior 38: Plugin registers new type reflections --

#[test]
fn plugin_registers_new_type_reflections() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
    app.update();

    let registry = app.world().resource::<AppTypeRegistry>();
    let guard = registry.read();

    assert!(
        guard.get(std::any::TypeId::of::<Velocity2D>()).is_some(),
        "Velocity2D should be registered for reflection"
    );
    assert!(
        guard
            .get(std::any::TypeId::of::<PreviousVelocity>())
            .is_some(),
        "PreviousVelocity should be registered for reflection"
    );
    assert!(
        guard
            .get(std::any::TypeId::of::<GlobalPosition2D>())
            .is_some(),
        "GlobalPosition2D should be registered for reflection"
    );
    assert!(
        guard
            .get(std::any::TypeId::of::<GlobalRotation2D>())
            .is_some(),
        "GlobalRotation2D should be registered for reflection"
    );
    assert!(
        guard.get(std::any::TypeId::of::<GlobalScale2D>()).is_some(),
        "GlobalScale2D should be registered for reflection"
    );
    drop(guard);
}
