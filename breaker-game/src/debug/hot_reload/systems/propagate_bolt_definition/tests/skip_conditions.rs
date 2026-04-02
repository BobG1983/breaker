use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Scale2D,
};

use super::helpers::*;
use crate::{
    bolt::components::{Bolt, BoltBaseDamage, BoltDefinitionRef},
    shared::size::BaseRadius,
};

// ---- Behavior 17 edge case: BoltRegistry change skips bolt with missing definition ----

#[test]
fn registry_change_skips_bolt_with_missing_definition() {
    let mut app = test_app();
    let initial = make_bolt_def(TEST_BOLT_NAME);
    seed_and_flush(&mut app, initial);

    // Spawn bolt referencing a name not in registry
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("MissingDef".to_owned()),
            BaseSpeed(999.0),
            MinSpeed(999.0),
            MaxSpeed(999.0),
            BaseRadius(999.0),
            BoltBaseDamage(999.0),
            Scale2D { x: 999.0, y: 999.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(999.0, 999.0)),
            MinAngleHorizontal(999.0),
            MinAngleVertical(999.0),
        ))
        .id();

    mutate_registry(&mut app, make_bolt_def(TEST_BOLT_NAME));
    app.update();

    // Bolt referencing "MissingDef" should be untouched
    let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
    assert!(
        (base_speed.0 - 999.0).abs() < f32::EPSILON,
        "bolt with missing definition should keep its original BaseSpeed 999.0, got {}",
        base_speed.0
    );
}

// ---- Behavior 23: System does NOT run when BoltRegistry is added (only on change) ----

#[test]
fn system_does_not_run_on_registry_added() {
    let mut app = test_app();

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(999.0),
            MinSpeed(999.0),
            MaxSpeed(999.0),
            BaseRadius(999.0),
            BoltBaseDamage(999.0),
            Scale2D { x: 999.0, y: 999.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(999.0, 999.0)),
            MinAngleHorizontal(999.0),
            MinAngleVertical(999.0),
        ))
        .id();

    // Insert registry (is_added() returns true on next update)
    {
        let mut registry = app
            .world_mut()
            .resource_mut::<crate::bolt::registry::BoltRegistry>();
        registry.insert(TEST_BOLT_NAME.to_owned(), make_bolt_def(TEST_BOLT_NAME));
    }

    // First update: registry is_added() is true, system should exit early
    app.update();

    let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
    assert!(
        (base_speed.0 - 999.0).abs() < f32::EPSILON,
        "BaseSpeed should remain 999.0 when registry is_added(), got {}",
        base_speed.0
    );
}

#[test]
fn system_does_not_run_on_subsequent_frame_without_mutation() {
    let mut app = test_app();

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(999.0),
            MinSpeed(999.0),
            MaxSpeed(999.0),
            BaseRadius(999.0),
            BoltBaseDamage(999.0),
            Scale2D { x: 999.0, y: 999.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(999.0, 999.0)),
            MinAngleHorizontal(999.0),
            MinAngleVertical(999.0),
        ))
        .id();

    {
        let mut registry = app
            .world_mut()
            .resource_mut::<crate::bolt::registry::BoltRegistry>();
        registry.insert(TEST_BOLT_NAME.to_owned(), make_bolt_def(TEST_BOLT_NAME));
    }

    // Two updates to clear is_added
    app.update();
    app.update();

    // Third update: no mutation, so is_changed() is false
    app.update();

    let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
    assert!(
        (base_speed.0 - 999.0).abs() < f32::EPSILON,
        "BaseSpeed should remain 999.0 when registry has not been mutated"
    );
}

// ---- Behavior 24: System handles zero bolt entities without panic ----

#[test]
fn system_handles_zero_bolt_entities() {
    let mut app = test_app();
    let initial = make_bolt_def(TEST_BOLT_NAME);
    seed_and_flush(&mut app, initial);

    // No bolt entities spawned
    mutate_registry(&mut app, make_bolt_def(TEST_BOLT_NAME));
    app.update();
    // Should not panic
}

#[test]
fn system_skips_entities_without_bolt_definition_ref() {
    let mut app = test_app();
    let initial = make_bolt_def(TEST_BOLT_NAME);
    seed_and_flush(&mut app, initial);

    // Spawn bolt WITHOUT BoltDefinitionRef
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BaseSpeed(999.0),
            MinSpeed(999.0),
            MaxSpeed(999.0),
            BaseRadius(999.0),
            BoltBaseDamage(999.0),
            Scale2D { x: 999.0, y: 999.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(999.0, 999.0)),
            MinAngleHorizontal(999.0),
            MinAngleVertical(999.0),
        ))
        .id();

    mutate_registry(&mut app, make_bolt_def(TEST_BOLT_NAME));
    app.update();

    // Entity without BoltDefinitionRef should be untouched
    let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
    assert!(
        (base_speed.0 - 999.0).abs() < f32::EPSILON,
        "bolt without BoltDefinitionRef should not be modified"
    );
}
