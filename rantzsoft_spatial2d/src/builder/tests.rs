use bevy::prelude::{Vec2, World};

use crate::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Position2D,
    PreviousPosition, Spatial,
};

#[test]
fn speed_only_builds_marker_and_base_speed() {
    let mut world = World::new();
    let entity = world
        .spawn(Spatial::builder().with_speed(400.0).build())
        .id();
    assert!(world.get::<Spatial>(entity).is_some());
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
}

#[test]
fn clamped_speed_builds_marker_base_min_max() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .with_clamped_speed(400.0, 200.0, 800.0)
                .build(),
        )
        .id();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let min = world.get::<MinSpeed>(entity).unwrap();
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
    assert!((min.0 - 200.0).abs() < f32::EPSILON);
    assert!((max.0 - 800.0).abs() < f32::EPSILON);
}

#[test]
fn speed_then_clamped_builds_same_as_clamped_speed() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .with_speed(400.0)
                .clamped(200.0, 800.0)
                .build(),
        )
        .id();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let min = world.get::<MinSpeed>(entity).unwrap();
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
    assert!((min.0 - 200.0).abs() < f32::EPSILON);
    assert!((max.0 - 800.0).abs() < f32::EPSILON);
}

#[test]
fn speed_with_angle_builds_marker_base_angles() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .with_speed(400.0)
                .with_clamped_angle(0.087, 0.087)
                .build(),
        )
        .id();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let angle_h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let angle_v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
    assert!((angle_h.0 - 0.087).abs() < f32::EPSILON);
    assert!((angle_v.0 - 0.087).abs() < f32::EPSILON);
}

#[test]
fn clamped_speed_with_angle_builds_full_tuple() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .with_clamped_speed(400.0, 200.0, 800.0)
                .with_clamped_angle(0.087, 0.087)
                .build(),
        )
        .id();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let min = world.get::<MinSpeed>(entity).unwrap();
    let max = world.get::<MaxSpeed>(entity).unwrap();
    let angle_h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let angle_v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!((base.0 - 400.0).abs() < f32::EPSILON);
    assert!((min.0 - 200.0).abs() < f32::EPSILON);
    assert!((max.0 - 800.0).abs() < f32::EPSILON);
    assert!((angle_h.0 - 0.087).abs() < f32::EPSILON);
    assert!((angle_v.0 - 0.087).abs() < f32::EPSILON);
}

#[test]
fn angle_before_speed_works() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .with_clamped_angle(0.1, 0.2)
                .with_clamped_speed(500.0, 100.0, 900.0)
                .build(),
        )
        .id();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let min = world.get::<MinSpeed>(entity).unwrap();
    let max = world.get::<MaxSpeed>(entity).unwrap();
    let angle_h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let angle_v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!((base.0 - 500.0).abs() < f32::EPSILON);
    assert!((min.0 - 100.0).abs() < f32::EPSILON);
    assert!((max.0 - 900.0).abs() < f32::EPSILON);
    assert!((angle_h.0 - 0.1).abs() < f32::EPSILON);
    assert!((angle_v.0 - 0.2).abs() < f32::EPSILON);
}

#[test]
fn no_position_uses_zero_default() {
    let mut world = World::new();
    let entity = world
        .spawn(Spatial::builder().with_speed(400.0).build())
        .id();
    let pos = world.get::<Position2D>(entity).unwrap();
    let prev = world.get::<PreviousPosition>(entity).unwrap();
    assert_eq!(pos.0, Vec2::ZERO);
    assert_eq!(prev.0, Vec2::ZERO);
}

#[test]
fn at_position_stores_position() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .at_position(Vec2::new(100.0, 200.0))
                .with_speed(400.0)
                .build(),
        )
        .id();
    let pos = world.get::<Position2D>(entity).unwrap();
    let prev = world.get::<PreviousPosition>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(100.0, 200.0));
    assert_eq!(prev.0, Vec2::new(100.0, 200.0));
}

#[test]
fn at_position_with_clamped_speed_and_angle() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .at_position(Vec2::new(50.0, 75.0))
                .with_clamped_speed(400.0, 200.0, 800.0)
                .with_clamped_angle(0.087, 0.087)
                .build(),
        )
        .id();
    let pos = world.get::<Position2D>(entity).unwrap();
    let prev = world.get::<PreviousPosition>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(50.0, 75.0));
    assert_eq!(prev.0, Vec2::new(50.0, 75.0));
}

#[test]
fn at_position_before_or_after_speed() {
    let mut world = World::new();
    // Position before speed
    let entity = world
        .spawn(
            Spatial::builder()
                .at_position(Vec2::new(10.0, 20.0))
                .with_speed(500.0)
                .build(),
        )
        .id();
    let pos = world.get::<Position2D>(entity).unwrap();
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(10.0, 20.0));
    assert!((base.0 - 500.0).abs() < f32::EPSILON);

    // Speed before position — can't do this since at_position is only on NoPosition
    // and speed transition doesn't change position state. Position must come
    // before or after speed, both work since at_position is on NoPosition for any S.
}

#[test]
fn at_position_negative_coordinates() {
    let mut world = World::new();
    let entity = world
        .spawn(
            Spatial::builder()
                .at_position(Vec2::new(-100.0, -50.0))
                .with_speed(400.0)
                .build(),
        )
        .id();
    let pos = world.get::<Position2D>(entity).unwrap();
    let prev = world.get::<PreviousPosition>(entity).unwrap();
    assert_eq!(pos.0, Vec2::new(-100.0, -50.0));
    assert_eq!(prev.0, Vec2::new(-100.0, -50.0));
}
