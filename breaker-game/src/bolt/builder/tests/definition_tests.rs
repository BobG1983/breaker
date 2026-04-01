use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, PreviousScale, Scale2D,
    Velocity2D,
};

use super::super::core::*;
use crate::bolt::{
    components::{
        Bolt, BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltInitialAngle, BoltLifespan,
        BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltSpawnOffsetY,
        SpawnedByEvolution,
    },
    definition::BoltDefinition,
    resources::BoltConfig,
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

// ── Behavior 34: .definition() does NOT insert removed components ──

#[test]
fn from_definition_does_not_insert_config_only_components() {
    let def = make_bolt_definition("Bolt", 10.0);
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);
    assert!(
        world.get::<BoltRespawnOffsetY>(entity).is_none(),
        "definition bolt should NOT have BoltRespawnOffsetY"
    );
    assert!(
        world.get::<BoltRespawnAngleSpread>(entity).is_none(),
        "definition bolt should NOT have BoltRespawnAngleSpread"
    );
    assert!(
        world.get::<BoltInitialAngle>(entity).is_none(),
        "definition bolt should NOT have BoltInitialAngle"
    );
}

// ── Behavior 35: .definition() and .config() produce equivalent speed, angle, radius ──

#[test]
fn definition_and_config_produce_equivalent_speed_angle_radius() {
    let config = BoltConfig {
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        radius: 14.0,
        ..BoltConfig::default()
    };
    let def = make_bolt_definition("Bolt", 10.0);

    let mut world = World::new();
    let config_entity = Bolt::builder()
        .config(&config)
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .spawn(&mut world);
    let def_entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .spawn(&mut world);

    // Speed
    let cfg_base = world.get::<BaseSpeed>(config_entity).unwrap();
    let def_base = world.get::<BaseSpeed>(def_entity).unwrap();
    assert!(
        (cfg_base.0 - def_base.0).abs() < f32::EPSILON,
        "BaseSpeed should match: config={}, definition={}",
        cfg_base.0,
        def_base.0
    );
    let cfg_min = world.get::<MinSpeed>(config_entity).unwrap();
    let def_min = world.get::<MinSpeed>(def_entity).unwrap();
    assert!(
        (cfg_min.0 - def_min.0).abs() < f32::EPSILON,
        "MinSpeed should match: config={}, definition={}",
        cfg_min.0,
        def_min.0
    );
    let cfg_max = world.get::<MaxSpeed>(config_entity).unwrap();
    let def_max = world.get::<MaxSpeed>(def_entity).unwrap();
    assert!(
        (cfg_max.0 - def_max.0).abs() < f32::EPSILON,
        "MaxSpeed should match: config={}, definition={}",
        cfg_max.0,
        def_max.0
    );

    // Angle
    let cfg_h = world.get::<MinAngleHorizontal>(config_entity).unwrap();
    let def_h = world.get::<MinAngleHorizontal>(def_entity).unwrap();
    assert!(
        (cfg_h.0 - def_h.0).abs() < 1e-5,
        "MinAngleHorizontal should match: config={}, definition={}",
        cfg_h.0,
        def_h.0
    );
    let cfg_v = world.get::<MinAngleVertical>(config_entity).unwrap();
    let def_v = world.get::<MinAngleVertical>(def_entity).unwrap();
    assert!(
        (cfg_v.0 - def_v.0).abs() < 1e-5,
        "MinAngleVertical should match: config={}, definition={}",
        cfg_v.0,
        def_v.0
    );

    // Radius
    let cfg_radius = world.get::<BoltRadius>(config_entity).unwrap();
    let def_radius = world.get::<BoltRadius>(def_entity).unwrap();
    assert!(
        (cfg_radius.0 - def_radius.0).abs() < f32::EPSILON,
        "BoltRadius should match: config={}, definition={}",
        cfg_radius.0,
        def_radius.0
    );
}

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

// ── Behavior 40: .definition() and .config() produce different bolt-param sets ──

#[test]
fn definition_and_config_produce_different_bolt_param_component_sets() {
    let def = make_bolt_definition("Bolt", 10.0);
    let config = BoltConfig::default();

    let mut world = World::new();

    // Entity A: via definition()
    let entity_a = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .serving()
        .primary()
        .spawn(&mut world);

    // Entity B: via config()
    let entity_b = Bolt::builder()
        .config(&config)
        .at_position(Vec2::new(100.0, 50.0))
        .serving()
        .primary()
        .spawn(&mut world);

    // Entity A (definition) should NOT have config-only components
    assert!(
        world.get::<BoltRespawnOffsetY>(entity_a).is_none(),
        "definition bolt should NOT have BoltRespawnOffsetY"
    );
    assert!(
        world.get::<BoltRespawnAngleSpread>(entity_a).is_none(),
        "definition bolt should NOT have BoltRespawnAngleSpread"
    );
    assert!(
        world.get::<BoltInitialAngle>(entity_a).is_none(),
        "definition bolt should NOT have BoltInitialAngle"
    );

    // Entity B (config) should HAVE config-only components
    let respawn_offset = world
        .get::<BoltRespawnOffsetY>(entity_b)
        .expect("config bolt should have BoltRespawnOffsetY");
    assert!(
        (respawn_offset.0 - 30.0).abs() < f32::EPSILON,
        "BoltRespawnOffsetY should be 30.0"
    );
    let respawn_angle = world
        .get::<BoltRespawnAngleSpread>(entity_b)
        .expect("config bolt should have BoltRespawnAngleSpread");
    assert!(
        (respawn_angle.0 - 0.524).abs() < f32::EPSILON,
        "BoltRespawnAngleSpread should be 0.524"
    );
    let initial_angle = world
        .get::<BoltInitialAngle>(entity_b)
        .expect("config bolt should have BoltInitialAngle");
    assert!(
        (initial_angle.0 - 0.26).abs() < f32::EPSILON,
        "BoltInitialAngle should be 0.26"
    );

    // Entity A (definition) should have definition-only components
    assert!(
        world.get::<BoltDefinitionRef>(entity_a).is_some(),
        "definition bolt should have BoltDefinitionRef"
    );
    assert!(
        world.get::<BoltBaseDamage>(entity_a).is_some(),
        "definition bolt should have BoltBaseDamage"
    );

    // Entity B (config) should NOT have definition-only components
    assert!(
        world.get::<BoltDefinitionRef>(entity_b).is_none(),
        "config bolt should NOT have BoltDefinitionRef"
    );
    assert!(
        world.get::<BoltBaseDamage>(entity_b).is_none(),
        "config bolt should NOT have BoltBaseDamage"
    );
}
