use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Scale2D,
};

use super::helpers::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltDefinitionRef, BoltRadius},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    shared::size::BaseRadius,
};

// ---- Behavior 22: All bolt entities updated, not just the first ----

#[test]
fn registry_change_updates_all_bolt_entities() {
    let mut app = test_app();
    let initial = make_bolt_def(TEST_BOLT_NAME);
    seed_and_flush(&mut app, initial);

    let spawn_bolt = |app: &mut App| -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
                BaseSpeed(100.0),
                MinSpeed(100.0),
                MaxSpeed(100.0),
                BaseRadius(100.0),
                BoltBaseDamage(100.0),
                Scale2D { x: 100.0, y: 100.0 },
                Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
                MinAngleHorizontal(100.0),
                MinAngleVertical(100.0),
            ))
            .id()
    };

    let bolt1 = spawn_bolt(&mut app);
    let bolt2 = spawn_bolt(&mut app);
    let bolt3 = spawn_bolt(&mut app);

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.base_speed = 720.0;
    mutate_registry(&mut app, updated);
    app.update();

    for (label, bolt) in [("bolt1", bolt1), ("bolt2", bolt2), ("bolt3", bolt3)] {
        let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
        assert!(
            (base_speed.0 - 720.0).abs() < f32::EPSILON,
            "{label} BaseSpeed should be 720.0, got {}",
            base_speed.0
        );
    }
}

#[test]
fn registry_change_updates_bolts_with_different_definitions() {
    let mut app = test_app();
    let test_def = make_bolt_def(TEST_BOLT_NAME);
    let mut heavy_def = make_bolt_def("HeavyBolt");
    heavy_def.base_speed = 500.0;

    {
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
        registry.insert(test_def.name.clone(), test_def);
        registry.insert(heavy_def.name.clone(), heavy_def);
    }
    app.update();
    app.update();

    let bolt_test = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(100.0),
            MinSpeed(100.0),
            MaxSpeed(100.0),
            BaseRadius(100.0),
            BoltBaseDamage(100.0),
            Scale2D { x: 100.0, y: 100.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            MinAngleHorizontal(100.0),
            MinAngleVertical(100.0),
        ))
        .id();
    let bolt_heavy = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("HeavyBolt".to_owned()),
            BaseSpeed(100.0),
            MinSpeed(100.0),
            MaxSpeed(100.0),
            BaseRadius(100.0),
            BoltBaseDamage(100.0),
            Scale2D { x: 100.0, y: 100.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            MinAngleHorizontal(100.0),
            MinAngleVertical(100.0),
        ))
        .id();

    // Mutate both definitions with new values
    let mut updated_test = make_bolt_def(TEST_BOLT_NAME);
    updated_test.base_speed = 720.0;
    let mut updated_heavy = make_bolt_def("HeavyBolt");
    updated_heavy.base_speed = 500.0;

    {
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
        registry.clear();
        registry.insert(updated_test.name.clone(), updated_test);
        registry.insert(updated_heavy.name.clone(), updated_heavy);
    }
    app.update();

    let test_speed = app.world().get::<BaseSpeed>(bolt_test).unwrap();
    assert!(
        (test_speed.0 - 720.0).abs() < f32::EPSILON,
        "TestBolt BaseSpeed should be 720.0, got {}",
        test_speed.0
    );
    let heavy_speed = app.world().get::<BaseSpeed>(bolt_heavy).unwrap();
    assert!(
        (heavy_speed.0 - 500.0).abs() < f32::EPSILON,
        "HeavyBolt BaseSpeed should be 500.0, got {}",
        heavy_speed.0
    );
}

// ---- Behavior 25: All definition-derived components re-stamped in a single propagation ----

#[test]
fn all_definition_derived_components_restamped_in_single_propagation() {
    let mut app = test_app();
    let initial = make_bolt_def(TEST_BOLT_NAME);
    seed_and_flush(&mut app, initial);

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

    let updated = BoltDefinition {
        name:                 TEST_BOLT_NAME.to_owned(),
        base_speed:           720.0,
        min_speed:            360.0,
        max_speed:            1440.0,
        radius:               14.0,
        base_damage:          10.0,
        effects:              vec![],
        color_rgb:            [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical:   5.0,
        min_radius:           None,
        max_radius:           None,
    };
    mutate_registry(&mut app, updated);
    app.update();

    let world = app.world();
    assert!(
        (world.get::<BaseSpeed>(bolt).unwrap().0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0"
    );
    assert!(
        (world.get::<MinSpeed>(bolt).unwrap().0 - 360.0).abs() < f32::EPSILON,
        "MinSpeed should be 360.0"
    );
    assert!(
        (world.get::<MaxSpeed>(bolt).unwrap().0 - 1440.0).abs() < f32::EPSILON,
        "MaxSpeed should be 1440.0"
    );
    assert!(
        (world.get::<BoltRadius>(bolt).unwrap().0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0"
    );
    assert!(
        (world.get::<BoltBaseDamage>(bolt).unwrap().0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0"
    );
    let scale = world.get::<Scale2D>(bolt).unwrap();
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON,
        "Scale2D.x should be 14.0"
    );
    assert!(
        (scale.y - 14.0).abs() < f32::EPSILON,
        "Scale2D.y should be 14.0"
    );
    let aabb = world.get::<Aabb2D>(bolt).unwrap();
    assert!(
        (aabb.half_extents.x - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.x should be 14.0"
    );
    assert!(
        (aabb.half_extents.y - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.y should be 14.0"
    );
    let expected_h = 5.0_f32.to_radians();
    let expected_v = 5.0_f32.to_radians();
    assert!(
        (world.get::<MinAngleHorizontal>(bolt).unwrap().0 - expected_h).abs() < 1e-5,
        "MinAngleHorizontal should be ~{expected_h} radians"
    );
    assert!(
        (world.get::<MinAngleVertical>(bolt).unwrap().0 - expected_v).abs() < 1e-5,
        "MinAngleVertical should be ~{expected_v} radians"
    );
}
