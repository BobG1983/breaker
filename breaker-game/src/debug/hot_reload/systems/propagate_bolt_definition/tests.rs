//! Tests for `propagate_bolt_definition` — behaviors 17-26.

use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Scale2D,
};

use super::system::propagate_bolt_definition;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltDefinitionRef, BoltRadius},
        definition::BoltDefinition,
        registry::BoltRegistry,
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
    shared::size::BaseRadius,
};

const TEST_BOLT_NAME: &str = "TestBolt";

/// Creates a minimal `BoltDefinition` with standard values.
fn make_bolt_def(name: &str) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Creates a test app with the `propagate_bolt_definition` system.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<BoltRegistry>()
        .add_systems(Update, propagate_bolt_definition);
    app
}

/// Seeds the registry with a definition and flushes the Added state.
///
/// Returns the app ready for mutation-triggered propagation.
fn seed_and_flush(app: &mut App, def: BoltDefinition) {
    {
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
        registry.insert(def.name.clone(), def);
    }
    // Flush Added change detection (two updates to clear is_added)
    app.update();
    app.update();
}

/// Mutates the registry by clearing and re-inserting a definition.
fn mutate_registry(app: &mut App, def: BoltDefinition) {
    let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
    registry.clear();
    registry.insert(def.name.clone(), def);
}

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
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
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
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
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
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
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

// ---- Behavior 26: Hot reload rebuilds definition-sourced BoundEffects, preserves chip-sourced ----

/// Seeds a 2-effect definition (`PerfectBumped` + `EarlyBumped`), spawns a bolt with matching
/// definition-sourced `BoundEffects` plus a chip-sourced piercing entry. Returns the bolt entity.
fn seed_two_effect_bolt_with_chip(app: &mut App) -> Entity {
    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(app, initial);

    app.world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            BoundEffects(vec![
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                ),
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::EarlyBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                    },
                ),
                (
                    "piercing_chip".to_owned(),
                    EffectNode::When {
                        trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
                    },
                ),
            ]),
            StagedEffects::default(),
        ))
        .id()
}

#[test]
fn hot_reload_rebuilds_definition_sourced_bound_effects_preserves_chip_sourced() {
    let mut app = test_app();
    let bolt = seed_two_effect_bolt_with_chip(&mut app);

    // Update definition: remove old 2 effects, add 1 new one
    let updated = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::LateBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "should have 2 entries: 1 chip-sourced (preserved) + 1 definition-sourced (new); got {}",
        bound.0.len()
    );

    // The chip-sourced entry should be preserved
    let chip_entries: Vec<_> = bound
        .0
        .iter()
        .filter(|(name, _)| !name.is_empty())
        .collect();
    assert_eq!(
        chip_entries.len(),
        1,
        "should have exactly 1 chip-sourced entry"
    );
    assert_eq!(chip_entries[0].0, "piercing_chip");

    // The definition-sourced entry should be the new one (LateBumped)
    let def_entries: Vec<_> = bound.0.iter().filter(|(name, _)| name.is_empty()).collect();
    assert_eq!(
        def_entries.len(),
        1,
        "should have exactly 1 definition-sourced entry (new)"
    );
    assert!(matches!(
        &def_entries[0].1,
        EffectNode::When {
            trigger: Trigger::LateBumped,
            ..
        }
    ));
}

#[test]
fn hot_reload_empty_definition_effects_clears_definition_entries_keeps_chip() {
    let mut app = test_app();

    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(&mut app, initial);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            BoundEffects(vec![
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                ),
                (
                    "piercing_chip".to_owned(),
                    EffectNode::When {
                        trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
                    },
                ),
            ]),
            StagedEffects::default(),
        ))
        .id();

    // Update definition to empty effects
    let updated = make_bolt_def(TEST_BOLT_NAME); // effects: vec![]
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "should have only the chip-sourced entry remaining"
    );
    assert_eq!(&bound.0[0].0, "piercing_chip");
}

#[test]
fn hot_reload_no_chip_sourced_entries_clears_all_then_adds_new() {
    let mut app = test_app();

    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(&mut app, initial);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            // Only definition-sourced entries (no chip-sourced)
            BoundEffects(vec![(
                String::new(),
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            )]),
            StagedEffects::default(),
        ))
        .id();

    // Update definition with different effect
    let updated = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::LateBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "should have 1 entry (old cleared, new added)"
    );
    assert!(
        bound.0[0].0.is_empty(),
        "entry should be definition-sourced (empty chip name)"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::LateBumped,
            ..
        }
    ));
}
