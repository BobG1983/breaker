use bevy::prelude::*;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Position2D, Velocity2D,
};

use super::super::core::*;
use crate::bolt::{
    components::{Bolt, ExtraBolt},
    resources::BoltConfig,
};

// ── Section A: Entry Point and Typestate Dimensions ──────────────────

// Behavior 1: Bolt::builder() returns a builder in the fully-unconfigured state
#[test]
fn bolt_new_returns_unconfigured_builder() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, NoRole> = Bolt::builder();
    // Type annotation compiles successfully — that is the assertion.
}

#[test]
fn bolt_new_twice_produces_independent_builders() {
    let builder_a = Bolt::builder();
    let builder_b = Bolt::builder();
    // Both builders are independent — modifying one does not affect the other.
    let _a = builder_a.at_position(Vec2::new(1.0, 2.0));
    let _b = builder_b.at_position(Vec2::new(3.0, 4.0));
}

// Behavior 2: .at_position() transitions Position dimension
#[test]
fn at_position_transitions_to_has_position() {
    let _builder: BoltBuilder<HasPosition, NoSpeed, NoAngle, NoMotion, NoRole> =
        Bolt::builder().at_position(Vec2::new(100.0, 250.0));
}

#[test]
fn at_position_stores_position_in_spawn() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::new(100.0, 250.0))
        .serving()
        .primary()
        .spawn(&mut world);
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - 100.0).abs() < f32::EPSILON && (pos.0.y - 250.0).abs() < f32::EPSILON,
        "Position2D should be (100.0, 250.0), got {:?}",
        pos.0
    );
}

#[test]
fn at_position_accepts_zero() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert_eq!(pos.0, Vec2::ZERO, "Position2D should be Vec2::ZERO");
}

#[test]
fn at_position_accepts_negative_coordinates() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::new(-200.0, -100.0))
        .serving()
        .primary()
        .spawn(&mut world);
    let pos = world
        .get::<Position2D>(entity)
        .expect("entity should have Position2D");
    assert!(
        (pos.0.x - (-200.0)).abs() < f32::EPSILON && (pos.0.y - (-100.0)).abs() < f32::EPSILON,
        "Position2D should be (-200.0, -100.0), got {:?}",
        pos.0
    );
}

// Behavior 3: .with_speed() transitions Speed dimension
#[test]
fn with_speed_transitions_to_has_speed() {
    let _builder: BoltBuilder<NoPosition, HasSpeed, NoAngle, NoMotion, NoRole> =
        Bolt::builder().with_speed(400.0, 200.0, 800.0);
}

#[test]
fn with_speed_stores_values_in_spawn() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world
        .get::<BaseSpeed>(entity)
        .expect("entity should have BaseSpeed");
    assert!(
        (base.0 - 400.0).abs() < f32::EPSILON,
        "BaseSpeed should be 400.0, got {}",
        base.0
    );
    let min = world
        .get::<MinSpeed>(entity)
        .expect("entity should have MinSpeed");
    assert!(
        (min.0 - 200.0).abs() < f32::EPSILON,
        "MinSpeed should be 200.0, got {}",
        min.0
    );
    let max = world
        .get::<MaxSpeed>(entity)
        .expect("entity should have MaxSpeed");
    assert!(
        (max.0 - 800.0).abs() < f32::EPSILON,
        "MaxSpeed should be 800.0, got {}",
        max.0
    );
}

#[test]
fn with_speed_equal_min_max_base_fixed_speed() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 400.0, 400.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world.get::<BaseSpeed>(entity).unwrap();
    let min = world.get::<MinSpeed>(entity).unwrap();
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (base.0 - 400.0).abs() < f32::EPSILON
            && (min.0 - 400.0).abs() < f32::EPSILON
            && (max.0 - 400.0).abs() < f32::EPSILON,
        "All speed values should be 400.0 for fixed-speed bolt"
    );
}

// Behavior 4: .with_angle() transitions Angle dimension
#[test]
fn with_angle_transitions_to_has_angle() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, HasAngle, NoMotion, NoRole> =
        Bolt::builder().with_angle(0.087, 0.087);
}

#[test]
fn with_angle_stores_values_in_spawn() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world
        .get::<MinAngleHorizontal>(entity)
        .expect("entity should have MinAngleHorizontal");
    assert!(
        (h.0 - 0.087).abs() < f32::EPSILON,
        "MinAngleHorizontal should be 0.087, got {}",
        h.0
    );
    let v = world
        .get::<MinAngleVertical>(entity)
        .expect("entity should have MinAngleVertical");
    assert!(
        (v.0 - 0.087).abs() < f32::EPSILON,
        "MinAngleVertical should be 0.087, got {}",
        v.0
    );
}

#[test]
fn with_angle_zero_valid() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.0, 0.0)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!(
        h.0.abs() < f32::EPSILON && v.0.abs() < f32::EPSILON,
        "Zero angles should produce MinAngleHorizontal(0.0) and MinAngleVertical(0.0)"
    );
}

// Behavior 5: .serving() transitions Motion dimension
#[test]
fn serving_transitions_to_serving() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, Serving, NoRole> =
        Bolt::builder().serving();
}

// Behavior 6: .with_velocity() transitions Motion dimension
#[test]
fn with_velocity_transitions_to_has_velocity() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, HasVelocity, NoRole> =
        Bolt::builder().with_velocity(Velocity2D(Vec2::new(102.9, 385.5)));
}

#[test]
fn with_velocity_stores_velocity_in_spawn() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::new(200.0, 300.0))
        .with_velocity(Velocity2D(Vec2::new(102.9, 385.5)))
        .extra()
        .spawn(&mut world);
    let vel = world
        .get::<Velocity2D>(entity)
        .expect("entity should have Velocity2D");
    assert!(
        (vel.0.x - 102.9).abs() < f32::EPSILON && (vel.0.y - 385.5).abs() < f32::EPSILON,
        "Velocity2D should be (102.9, 385.5), got {:?}",
        vel.0
    );
}

#[test]
fn with_velocity_zero_valid() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .config(&BoltConfig::default())
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::ZERO))
        .extra()
        .spawn(&mut world);
    // Must also check a non-#[require] component to avoid false pass from stub
    assert!(
        world.get::<ExtraBolt>(entity).is_some(),
        "entity should have ExtraBolt marker"
    );
    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "Velocity2D(Vec2::ZERO) should be valid for non-serving bolt"
    );
}

// Behavior 7: .primary() transitions Role dimension
#[test]
fn as_primary_transitions_to_primary() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, Primary> =
        Bolt::builder().primary();
}

// Behavior 8: .extra() transitions Role dimension
#[test]
fn as_extra_transitions_to_extra() {
    let _builder: BoltBuilder<NoPosition, NoSpeed, NoAngle, NoMotion, Extra> =
        Bolt::builder().extra();
}
