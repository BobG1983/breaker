use bevy::prelude::*;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed,
};

use super::helpers::*;
use crate::{
    bolt::components::{BoltBaseDamage, BoltDefinitionRef, BoltRadius},
    prelude::*,
    shared::size::BaseRadius,
};

// ---- Behavior 17: BoltRegistry change re-stamps BaseSpeed ----

#[test]
fn registry_change_restamps_base_speed() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.base_speed = 720.0;
    mutate_registry(&mut app, updated);
    app.update();

    let base_speed = app.world().get::<BaseSpeed>(bolt).unwrap();
    assert!(
        (base_speed.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0 after registry change, got {}",
        base_speed.0
    );
}

// ---- Behavior 18: BoltRegistry change re-stamps MinSpeed and MaxSpeed ----

#[test]
fn registry_change_restamps_min_speed_and_max_speed() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.min_speed = 360.0;
    updated.max_speed = 1440.0;
    mutate_registry(&mut app, updated);
    app.update();

    let min_speed = app.world().get::<MinSpeed>(bolt).unwrap();
    assert!(
        (min_speed.0 - 360.0).abs() < f32::EPSILON,
        "MinSpeed should be 360.0, got {}",
        min_speed.0
    );
    let max_speed = app.world().get::<MaxSpeed>(bolt).unwrap();
    assert!(
        (max_speed.0 - 1440.0).abs() < f32::EPSILON,
        "MaxSpeed should be 1440.0, got {}",
        max_speed.0
    );
}

#[test]
fn registry_change_handles_equal_min_and_max_speed() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.min_speed = 720.0;
    updated.max_speed = 720.0;
    mutate_registry(&mut app, updated);
    app.update();

    let min_speed = app.world().get::<MinSpeed>(bolt).unwrap();
    let max_speed = app.world().get::<MaxSpeed>(bolt).unwrap();
    assert!(
        (min_speed.0 - 720.0).abs() < f32::EPSILON,
        "MinSpeed should be 720.0 when equal to MaxSpeed"
    );
    assert!(
        (max_speed.0 - 720.0).abs() < f32::EPSILON,
        "MaxSpeed should be 720.0 when equal to MinSpeed"
    );
}

// ---- Behavior 19: BoltRegistry change re-stamps BoltRadius, Scale2D, Aabb2D ----

#[test]
fn registry_change_restamps_radius_scale_and_aabb() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.radius = 14.0;
    mutate_registry(&mut app, updated);
    app.update();

    let bolt_radius = app.world().get::<BoltRadius>(bolt).unwrap();
    assert!(
        (bolt_radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0, got {}",
        bolt_radius.0
    );

    let scale = app.world().get::<Scale2D>(bolt).unwrap();
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON,
        "Scale2D.x should be 14.0, got {}",
        scale.x
    );
    assert!(
        (scale.y - 14.0).abs() < f32::EPSILON,
        "Scale2D.y should be 14.0, got {}",
        scale.y
    );

    let aabb = app.world().get::<Aabb2D>(bolt).unwrap();
    assert!(
        (aabb.half_extents.x - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.x should be 14.0, got {}",
        aabb.half_extents.x
    );
    assert!(
        (aabb.half_extents.y - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.y should be 14.0, got {}",
        aabb.half_extents.y
    );
}

#[test]
fn registry_change_handles_very_small_radius() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.radius = 1.0;
    mutate_registry(&mut app, updated);
    app.update();

    let bolt_radius = app.world().get::<BoltRadius>(bolt).unwrap();
    assert!(
        (bolt_radius.0 - 1.0).abs() < f32::EPSILON,
        "BoltRadius should be 1.0 for very small radius"
    );
    let scale = app.world().get::<Scale2D>(bolt).unwrap();
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON,
        "Scale2D.x should be 1.0 for very small radius"
    );
    let aabb = app.world().get::<Aabb2D>(bolt).unwrap();
    assert!(
        (aabb.half_extents.x - 1.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be 1.0 for very small radius"
    );
}

// ---- Behavior 20: BoltRegistry change re-stamps BoltBaseDamage ----

#[test]
fn registry_change_restamps_base_damage() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.base_damage = 10.0;
    mutate_registry(&mut app, updated);
    app.update();

    let damage = app.world().get::<BoltBaseDamage>(bolt).unwrap();
    assert!(
        (damage.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        damage.0
    );
}

#[test]
fn registry_change_handles_zero_base_damage() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.base_damage = 0.0;
    mutate_registry(&mut app, updated);
    app.update();

    let damage = app.world().get::<BoltBaseDamage>(bolt).unwrap();
    assert!(
        (damage.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0 when definition has zero damage"
    );
}

// ---- Behavior 21: BoltRegistry change re-stamps angle clamping components ----

#[test]
fn registry_change_restamps_angle_clamping_in_radians() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.min_angle_horizontal = 5.0;
    updated.min_angle_vertical = 5.0;
    mutate_registry(&mut app, updated);
    app.update();

    let expected_radians = 5.0_f32.to_radians();
    let h = app.world().get::<MinAngleHorizontal>(bolt).unwrap();
    assert!(
        (h.0 - expected_radians).abs() < 1e-5,
        "MinAngleHorizontal should be ~{expected_radians} radians, got {}",
        h.0
    );
    let v = app.world().get::<MinAngleVertical>(bolt).unwrap();
    assert!(
        (v.0 - expected_radians).abs() < 1e-5,
        "MinAngleVertical should be ~{expected_radians} radians, got {}",
        v.0
    );
}

#[test]
fn registry_change_handles_zero_angle_clamping() {
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

    let mut updated = make_bolt_def(TEST_BOLT_NAME);
    updated.min_angle_horizontal = 0.0;
    updated.min_angle_vertical = 0.0;
    mutate_registry(&mut app, updated);
    app.update();

    let h = app.world().get::<MinAngleHorizontal>(bolt).unwrap();
    assert!(
        h.0.abs() < f32::EPSILON,
        "MinAngleHorizontal should be 0.0 radians when definition has 0.0"
    );
}
