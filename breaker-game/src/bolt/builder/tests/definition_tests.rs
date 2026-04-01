use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, PreviousScale, Scale2D,
    Velocity2D,
};

use super::super::core::*;
use crate::bolt::{
    components::{
        Bolt, BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltLifespan, BoltRadius,
        BoltSpawnOffsetY, SpawnedByEvolution,
    },
    definition::BoltDefinition,
};

/// Creates a default `BoltDefinition` for test usage.
fn make_bolt_definition(name: &str, base_damage: f32) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
    }
}

// ── Behavior 26: .definition() satisfies Speed + Angle typestates ──

#[test]
fn from_definition_transitions_speed_and_angle() {
    let def = make_bolt_definition("Bolt", 10.0);
    let _builder: BoltBuilder<NoPosition, HasSpeed, HasAngle, NoMotion, NoRole> =
        Bolt::builder().definition(&def);
}

// ── Behavior 27: .definition() sets speed components from BoltDefinition fields ──

#[test]
fn from_definition_sets_speed_components() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed from definition should be 720.0, got {}",
        base.0
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 360.0).abs() < f32::EPSILON,
        "MinSpeed from definition should be 360.0, got {}",
        min.0
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 1440.0).abs() < f32::EPSILON,
        "MaxSpeed from definition should be 1440.0, got {}",
        max.0
    );
}

#[test]
fn from_definition_custom_speed_values_propagate() {
    let def = BoltDefinition {
        name: "Custom".to_string(),
        base_speed: 500.0,
        min_speed: 100.0,
        max_speed: 900.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let base = world.get::<BaseSpeed>(entity).unwrap();
    assert!(
        (base.0 - 500.0).abs() < f32::EPSILON,
        "BaseSpeed from custom definition should be 500.0, got {}",
        base.0
    );
    let min = world.get::<MinSpeed>(entity).unwrap();
    assert!(
        (min.0 - 100.0).abs() < f32::EPSILON,
        "MinSpeed from custom definition should be 100.0, got {}",
        min.0
    );
    let max = world.get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max.0 - 900.0).abs() < f32::EPSILON,
        "MaxSpeed from custom definition should be 900.0, got {}",
        max.0
    );
}

// ── Behavior 28: .definition() converts angle degrees to radians ──

#[test]
fn from_definition_converts_angles_to_radians() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    let expected_h = 5.0_f32.to_radians();
    assert!(
        (h.0 - expected_h).abs() < 1e-5,
        "MinAngleHorizontal should be {} (5 degrees in radians), got {}",
        expected_h,
        h.0
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    let expected_v = 5.0_f32.to_radians();
    assert!(
        (v.0 - expected_v).abs() < 1e-5,
        "MinAngleVertical should be {} (5 degrees in radians), got {}",
        expected_v,
        v.0
    );
}

#[test]
fn from_definition_zero_angles_produce_zero_radians() {
    let def = BoltDefinition {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..make_bolt_definition("Bolt", 10.0)
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let h = world.get::<MinAngleHorizontal>(entity).unwrap();
    assert!(
        h.0.abs() < f32::EPSILON,
        "MinAngleHorizontal(0.0) should be 0.0, got {}",
        h.0
    );
    let v = world.get::<MinAngleVertical>(entity).unwrap();
    assert!(
        v.0.abs() < f32::EPSILON,
        "MinAngleVertical(0.0) should be 0.0, got {}",
        v.0
    );
}

// ── Behavior 29: .definition() sets radius from BoltDefinition ──

#[test]
fn from_definition_sets_radius_and_physical_dimensions() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0, got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON && (scale.y - 14.0).abs() < f32::EPSILON,
        "Scale2D should be (14.0, 14.0), got ({}, {})",
        scale.x,
        scale.y
    );
    let prev_scale = world.get::<PreviousScale>(entity).unwrap();
    assert!(
        (prev_scale.x - 14.0).abs() < f32::EPSILON && (prev_scale.y - 14.0).abs() < f32::EPSILON,
        "PreviousScale should be (14.0, 14.0)"
    );
    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 14.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (14.0, 14.0)"
    );
}

#[test]
fn from_definition_custom_radius_propagates() {
    let def = BoltDefinition {
        radius: 7.0,
        ..make_bolt_definition("Small", 10.0)
    };
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 7.0).abs() < f32::EPSILON,
        "BoltRadius should be 7.0, got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 7.0).abs() < f32::EPSILON && (scale.y - 7.0).abs() < f32::EPSILON,
        "Scale2D should be (7.0, 7.0)"
    );
}

// ── Behavior 30: .definition() inserts BoltBaseDamage from base_damage ──

#[test]
fn from_definition_inserts_bolt_base_damage() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("definition bolt should have BoltBaseDamage");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        dmg.0
    );
}

#[test]
fn from_definition_zero_damage_inserts_bolt_base_damage_zero() {
    let def = make_bolt_definition("Zero", 0.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("zero damage bolt should still have BoltBaseDamage");
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0, got {}",
        dmg.0
    );
}

// ── Behavior 31: .definition() inserts BoltDefinitionRef with name ──

#[test]
fn from_definition_inserts_bolt_definition_ref() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("definition bolt should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "Bolt",
        "BoltDefinitionRef should be 'Bolt', got '{}'",
        def_ref.0
    );
}

#[test]
fn from_definition_heavy_inserts_bolt_definition_ref_heavy() {
    let def = make_bolt_definition("Heavy", 25.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("definition bolt should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "Heavy",
        "BoltDefinitionRef should be 'Heavy', got '{}'",
        def_ref.0
    );
}

// ── Behavior 32: .definition() inserts BoltAngleSpread with DEFAULT_BOLT_ANGLE_SPREAD ──

#[test]
fn from_definition_inserts_bolt_angle_spread() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("definition bolt should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.524 (DEFAULT_BOLT_ANGLE_SPREAD), got {}",
        spread.0
    );
}

// ── Behavior 33: .definition() inserts BoltSpawnOffsetY with DEFAULT_BOLT_SPAWN_OFFSET_Y ──

#[test]
fn from_definition_inserts_bolt_spawn_offset_y() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("definition bolt should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 54.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 54.0 (DEFAULT_BOLT_SPAWN_OFFSET_Y), got {}",
        offset.0
    );
}

// ── Behavior 34: .definition() does NOT insert config-only components ──
// (BoltRespawnOffsetY, BoltRespawnAngleSpread, BoltInitialAngle were deleted in Wave 6)

// Behavior 35 was deleted in Wave 6 (.config() no longer exists)

// ── Behavior 36: .definition() works with all motion/role combinations ──

#[test]
fn from_definition_works_with_all_motion_role_combinations() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();

    // serving + primary
    let sp = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);

    // serving + extra
    let se = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(10.0, 0.0))
        .serving()
        .extra()
        .spawn(&mut world);

    // velocity + primary
    let vp = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(20.0, 0.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 720.0)))
        .primary()
        .spawn(&mut world);

    // velocity + extra
    let ve = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(30.0, 0.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 720.0)))
        .extra()
        .spawn(&mut world);

    for (label, entity) in [("sp", sp), ("se", se), ("vp", vp), ("ve", ve)] {
        assert!(
            world.get_entity(entity).is_ok(),
            "{label}: entity should exist"
        );
        let dmg = world
            .get::<BoltBaseDamage>(entity)
            .unwrap_or_else(|| panic!("{label}: should have BoltBaseDamage"));
        assert!(
            (dmg.0 - 10.0).abs() < f32::EPSILON,
            "{label}: BoltBaseDamage should be 10.0, got {}",
            dmg.0
        );
        let def_ref = world
            .get::<BoltDefinitionRef>(entity)
            .unwrap_or_else(|| panic!("{label}: should have BoltDefinitionRef"));
        assert_eq!(
            def_ref.0, "Bolt",
            "{label}: BoltDefinitionRef should be 'Bolt'"
        );
        let spread = world
            .get::<BoltAngleSpread>(entity)
            .unwrap_or_else(|| panic!("{label}: should have BoltAngleSpread"));
        assert!(
            (spread.0 - 0.524).abs() < f32::EPSILON,
            "{label}: BoltAngleSpread should be 0.524, got {}",
            spread.0
        );
        let offset = world
            .get::<BoltSpawnOffsetY>(entity)
            .unwrap_or_else(|| panic!("{label}: should have BoltSpawnOffsetY"));
        assert!(
            (offset.0 - 54.0).abs() < f32::EPSILON,
            "{label}: BoltSpawnOffsetY should be 54.0, got {}",
            offset.0
        );
    }
}

// ── Behavior 37: .definition() combined with .with_radius() -- radius override wins ──

#[test]
fn from_definition_with_radius_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0); // radius 14.0 in definition
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_radius(20.0)
        .spawn(&mut world);
    let radius = world.get::<BoltRadius>(entity).unwrap();
    assert!(
        (radius.0 - 20.0).abs() < f32::EPSILON,
        "BoltRadius should be 20.0 (override wins), got {}",
        radius.0
    );
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 20.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D should be (20.0, 20.0)"
    );
}

// ── Behavior 38: .definition() combined with .spawned_by() ──

#[test]
fn from_definition_with_spawned_by_both_present() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 720.0)))
        .extra()
        .spawned_by("chain_bolt")
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("should have BoltDefinitionRef");
    assert_eq!(def_ref.0, "Bolt");
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("should have BoltBaseDamage");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let spawned_by = world
        .get::<SpawnedByEvolution>(entity)
        .expect("should have SpawnedByEvolution");
    assert_eq!(spawned_by.0, "chain_bolt");
}

// ── Behavior 39: .definition() combined with .with_lifespan() ──

#[test]
fn from_definition_with_lifespan_both_present() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 720.0)))
        .extra()
        .with_lifespan(3.0)
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("should have BoltDefinitionRef");
    assert_eq!(def_ref.0, "Bolt");
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("should have BoltBaseDamage");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let lifespan = world
        .get::<BoltLifespan>(entity)
        .expect("should have BoltLifespan");
    assert!(
        (lifespan.0.duration().as_secs_f32() - 3.0).abs() < 1e-3,
        "BoltLifespan duration should be ~3.0"
    );
}

// Behavior 40 was deleted in Wave 6 (.config() no longer exists)

// ── Section A: .with_base_damage() standalone ──────────────────────────────

// ── Behavior 1: .with_base_damage() inserts BoltBaseDamage on spawned entity ──

#[test]
fn with_base_damage_inserts_bolt_base_damage() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(15.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 15.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 15.0, got {}",
        dmg.0
    );
}

#[test]
fn with_base_damage_zero_is_valid() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(0.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage with zero value");
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0, got {}",
        dmg.0
    );
}

// ── Behavior 2: .with_base_damage() does not insert other definition components ──

#[test]
fn with_base_damage_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(15.0)
        .spawn(&mut world);
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_base_damage() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_base_damage() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_base_damage() was called"
    );
}

// ── Section B: .with_definition_name() standalone ──────────────────────────

// ── Behavior 3: .with_definition_name() inserts BoltDefinitionRef on spawned entity ──

#[test]
fn with_definition_name_inserts_bolt_definition_ref() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_definition_name("CustomBolt".to_string())
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "CustomBolt",
        "BoltDefinitionRef should be 'CustomBolt', got '{}'",
        def_ref.0
    );
}

#[test]
fn with_definition_name_empty_string_accepted() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_definition_name(String::new())
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef with empty string");
    assert_eq!(
        def_ref.0, "",
        "BoltDefinitionRef should be empty, got '{}'",
        def_ref.0
    );
}

// ── Behavior 4: .with_definition_name() does not insert other definition components ──

#[test]
fn with_definition_name_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_definition_name("CustomBolt".to_string())
        .spawn(&mut world);
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_definition_name() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_definition_name() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_definition_name() was called"
    );
}

// ── Section C: .with_angle_spread() standalone ─────────────────────────────

// ── Behavior 5: .with_angle_spread() inserts BoltAngleSpread on spawned entity ──

#[test]
fn with_angle_spread_inserts_bolt_angle_spread() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_angle_spread(0.35)
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.35).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.35, got {}",
        spread.0
    );
}

#[test]
fn with_angle_spread_zero_is_valid() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_angle_spread(0.0)
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread with zero value");
    assert!(
        spread.0.abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.0, got {}",
        spread.0
    );
}

// ── Behavior 6: .with_angle_spread() does not insert other definition components ──

#[test]
fn with_angle_spread_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_angle_spread(0.35)
        .spawn(&mut world);
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_angle_spread() was called"
    );
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_angle_spread() was called"
    );
    assert!(
        world.get::<BoltSpawnOffsetY>(entity).is_none(),
        "entity should NOT have BoltSpawnOffsetY when only .with_angle_spread() was called"
    );
}

// ── Section D: .with_spawn_offset_y() standalone ───────────────────────────

// ── Behavior 7: .with_spawn_offset_y() inserts BoltSpawnOffsetY on spawned entity ──

#[test]
fn with_spawn_offset_y_inserts_bolt_spawn_offset_y() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_spawn_offset_y(40.0)
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 40.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 40.0, got {}",
        offset.0
    );
}

#[test]
fn with_spawn_offset_y_zero_is_valid() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_spawn_offset_y(0.0)
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY with zero value");
    assert!(
        offset.0.abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 0.0, got {}",
        offset.0
    );
}

// ── Behavior 8: .with_spawn_offset_y() does not insert other definition components ──

#[test]
fn with_spawn_offset_y_does_not_insert_other_definition_components() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_spawn_offset_y(40.0)
        .spawn(&mut world);
    assert!(
        world.get::<BoltBaseDamage>(entity).is_none(),
        "entity should NOT have BoltBaseDamage when only .with_spawn_offset_y() was called"
    );
    assert!(
        world.get::<BoltDefinitionRef>(entity).is_none(),
        "entity should NOT have BoltDefinitionRef when only .with_spawn_offset_y() was called"
    );
    assert!(
        world.get::<BoltAngleSpread>(entity).is_none(),
        "entity should NOT have BoltAngleSpread when only .with_spawn_offset_y() was called"
    );
}

// ── Section E: Override semantics — .with_*() after .definition() ──────────

// ── Behavior 9: .with_base_damage() overrides definition's base_damage ──

#[test]
fn with_base_damage_overrides_definition_base_damage() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(25.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 25.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 25.0 (override), not 10.0 (definition), got {}",
        dmg.0
    );
    // Other definition fields remain intact
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.524 from definition, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!(
        (offset.0 - 54.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 54.0 from definition, got {}",
        offset.0
    );
}

#[test]
fn with_base_damage_zero_overrides_definition_base_damage() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(0.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        dmg.0.abs() < f32::EPSILON,
        "BoltBaseDamage should be 0.0 (zero override wins), got {}",
        dmg.0
    );
}

// ── Behavior 10: .with_definition_name() overrides definition's name ──

#[test]
fn with_definition_name_overrides_definition_name() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_definition_name("OverrideName".to_string())
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "OverrideName",
        "BoltDefinitionRef should be 'OverrideName' (override), not 'Bolt' (definition), got '{}'",
        def_ref.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0 from definition, got {}",
        dmg.0
    );
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!(
        (spread.0 - 0.524).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.524 from definition, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!(
        (offset.0 - 54.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 54.0 from definition, got {}",
        offset.0
    );
}

#[test]
fn with_definition_name_empty_overrides_definition_name() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_definition_name(String::new())
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "",
        "BoltDefinitionRef should be '' (empty override wins), got '{}'",
        def_ref.0
    );
}

// ── Behavior 11: .with_angle_spread() overrides definition's default angle spread ──

#[test]
fn with_angle_spread_overrides_definition_angle_spread() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_angle_spread(1.0)
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 1.0).abs() < f32::EPSILON,
        "BoltAngleSpread should be 1.0 (override), not 0.524 (definition), got {}",
        spread.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should still have BoltSpawnOffsetY from definition");
    assert!((offset.0 - 54.0).abs() < f32::EPSILON);
}

#[test]
fn with_angle_spread_zero_overrides_definition_angle_spread() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_angle_spread(0.0)
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        spread.0.abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.0 (zero override wins), got {}",
        spread.0
    );
}

// ── Behavior 12: .with_spawn_offset_y() overrides definition's default spawn offset ──

#[test]
fn with_spawn_offset_y_overrides_definition_spawn_offset() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_spawn_offset_y(30.0)
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 30.0 (override), not 54.0 (definition), got {}",
        offset.0
    );
    // Other definition fields remain intact
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should still have BoltBaseDamage from definition");
    assert!((dmg.0 - 10.0).abs() < f32::EPSILON);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should still have BoltDefinitionRef from definition");
    assert_eq!(def_ref.0, "Bolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should still have BoltAngleSpread from definition");
    assert!((spread.0 - 0.524).abs() < f32::EPSILON);
}

#[test]
fn with_spawn_offset_y_zero_overrides_definition_spawn_offset() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_spawn_offset_y(0.0)
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        offset.0.abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 0.0 (zero override wins), got {}",
        offset.0
    );
}

// ── Section F: All four .with_*() methods combined (no definition) ─────────

// ── Behavior 13: All four .with_*() methods together replace .definition() for definition-related fields ──

#[test]
fn all_four_with_methods_together_without_definition() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_speed(500.0, 250.0, 1000.0)
        .with_angle(0.1, 0.1)
        .at_position(Vec2::new(100.0, 200.0))
        .with_velocity(Velocity2D(Vec2::new(0.0, 500.0)))
        .extra()
        .with_base_damage(20.0)
        .with_definition_name("SyntheticBolt".to_string())
        .with_angle_spread(0.6)
        .with_spawn_offset_y(45.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 20.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 20.0, got {}",
        dmg.0
    );
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(def_ref.0, "SyntheticBolt");
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.6).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.6, got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 45.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 45.0, got {}",
        offset.0
    );
}

#[test]
fn all_four_with_methods_plus_definition_overrides_win() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(99.0)
        .with_definition_name("AllOverride".to_string())
        .with_angle_spread(2.0)
        .with_spawn_offset_y(100.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 99.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 99.0 (override), got {}",
        dmg.0
    );
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "AllOverride",
        "BoltDefinitionRef should be 'AllOverride' (override), got '{}'",
        def_ref.0
    );
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 2.0).abs() < f32::EPSILON,
        "BoltAngleSpread should be 2.0 (override), got {}",
        spread.0
    );
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 100.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 100.0 (override), got {}",
        offset.0
    );
}

// ── Section G: Optional method ordering and typestate independence ──────────

// ── Behaviors 14-17: .with_*() methods available in initial (unconfigured) typestate ──

#[test]
fn with_base_damage_available_in_initial_state() {
    // Verifies that .with_base_damage() compiles from initial state and
    // can continue chaining through to spawn.
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_base_damage(10.0)
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage when called early in chain");
    assert!(
        (dmg.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        dmg.0
    );
}

#[test]
fn with_definition_name_available_in_initial_state() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_definition_name("Test".to_string())
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef when called early in chain");
    assert_eq!(def_ref.0, "Test");
}

#[test]
fn with_angle_spread_available_in_initial_state() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_angle_spread(0.5)
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread when called early in chain");
    assert!(
        (spread.0 - 0.5).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.5, got {}",
        spread.0
    );
}

#[test]
fn with_spawn_offset_y_available_in_initial_state() {
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_spawn_offset_y(50.0)
        .with_speed(400.0, 200.0, 800.0)
        .with_angle(0.087, 0.087)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY when called early in chain");
    assert!(
        (offset.0 - 50.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 50.0, got {}",
        offset.0
    );
}

// ── Section H: Override ordering — .with_*() before .definition() ──────────

// ── Behavior 18: .with_base_damage() called before .definition() — explicit override still wins ──

#[test]
fn with_base_damage_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_base_damage(25.0)
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 25.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 25.0 (explicit override wins regardless of call order), got {}",
        dmg.0
    );
}

#[test]
fn with_base_damage_before_and_after_definition_last_with_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_base_damage(25.0)
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .with_base_damage(42.0)
        .spawn(&mut world);
    let dmg = world
        .get::<BoltBaseDamage>(entity)
        .expect("entity should have BoltBaseDamage");
    assert!(
        (dmg.0 - 42.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 42.0 (last .with_base_damage() wins), got {}",
        dmg.0
    );
}

// ── Behavior 19: .with_definition_name() called before .definition() — explicit override still wins ──

#[test]
fn with_definition_name_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_definition_name("Override".to_string())
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("entity should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "Override",
        "BoltDefinitionRef should be 'Override' (explicit override wins regardless of call order), got '{}'",
        def_ref.0
    );
}

// ── Behavior 20: .with_angle_spread() called before .definition() — explicit override still wins ──

#[test]
fn with_angle_spread_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_angle_spread(0.8)
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("entity should have BoltAngleSpread");
    assert!(
        (spread.0 - 0.8).abs() < f32::EPSILON,
        "BoltAngleSpread should be 0.8 (explicit override wins regardless of call order), got {}",
        spread.0
    );
}

// ── Behavior 21: .with_spawn_offset_y() called before .definition() — explicit override still wins ──

#[test]
fn with_spawn_offset_y_before_definition_explicit_override_wins() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .with_spawn_offset_y(30.0)
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    let offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("entity should have BoltSpawnOffsetY");
    assert!(
        (offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be 30.0 (explicit override wins regardless of call order), got {}",
        offset.0
    );
}
