//! Migration tests for `spawn_bolt` — verifying the system reads from
//! `BoltRegistry`/`BreakerRegistry`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{super::*, helpers::*};
use crate::{
    bolt::{
        components::{
            Bolt, BoltAngleSpread, BoltBaseDamage, BoltDefinitionRef, BoltRadius, BoltServing,
            BoltSpawnOffsetY,
        },
        definition::BoltDefinition,
        messages::BoltSpawned,
        registry::BoltRegistry,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    breaker::{
        components::Breaker, definition::BreakerDefinition, registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    run::RunState,
    shared::GameDrawLayer,
};

// ── Behavior 1: spawn_bolt reads BoltRegistry + BreakerRegistry + SelectedBreaker ──

#[test]
fn spawn_bolt_reads_registries_for_bolt_definition_components() {
    // Given: BoltRegistry with "Bolt" def (base_speed: 720.0, base_damage: 10.0),
    //        BreakerRegistry with "Aegis" (bolt: "Bolt"), SelectedBreaker("Aegis").
    //        No existing bolt. RunState.node_index == 0. Breaker at (0.0, -250.0).
    // When: spawn_bolt runs
    // Then: Bolt has BoltDefinitionRef("Bolt"), BoltBaseDamage(10.0),
    //       BoltAngleSpread(0.524), BoltSpawnOffsetY(54.0), BoltServing,
    //       Velocity2D(ZERO), position at (0.0, -196.0).
    let mut app = test_app_with_registries();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should be spawned");

    let world = app.world();

    let def_ref = world
        .get::<BoltDefinitionRef>(entity)
        .expect("bolt should have BoltDefinitionRef");
    assert_eq!(
        def_ref.0, "Bolt",
        "BoltDefinitionRef should be 'Bolt', got '{}'",
        def_ref.0
    );

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .expect("bolt should have BoltBaseDamage");
    assert!(
        (base_damage.0 - 10.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 10.0, got {}",
        base_damage.0
    );

    let angle_spread = world
        .get::<BoltAngleSpread>(entity)
        .expect("bolt should have BoltAngleSpread");
    assert!(
        (angle_spread.0 - DEFAULT_BOLT_ANGLE_SPREAD).abs() < f32::EPSILON,
        "BoltAngleSpread should be {DEFAULT_BOLT_ANGLE_SPREAD}, got {}",
        angle_spread.0
    );

    let spawn_offset = world
        .get::<BoltSpawnOffsetY>(entity)
        .expect("bolt should have BoltSpawnOffsetY");
    assert!(
        (spawn_offset.0 - DEFAULT_BOLT_SPAWN_OFFSET_Y).abs() < f32::EPSILON,
        "BoltSpawnOffsetY should be {DEFAULT_BOLT_SPAWN_OFFSET_Y}, got {}",
        spawn_offset.0
    );

    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "bolt on node_index 0 should have BoltServing"
    );

    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        vel.speed() < f32::EPSILON,
        "serving bolt velocity should be zero, got {}",
        vel.speed()
    );

    let pos = world.get::<Position2D>(entity).unwrap();
    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.x).abs() < f32::EPSILON,
        "bolt x should be 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "bolt y should be {expected_y}, got {}",
        pos.0.y
    );
}

// ── Behavior 2: spawn_bolt uses definition color for bolt material ──

#[test]
fn spawn_bolt_uses_definition_color_for_material() {
    // Given: BoltDefinition.color_rgb: [6.0, 5.0, 0.5]
    // When: spawn_bolt runs
    // Then: MeshMaterial2d color matches Color::linear_rgb(6.0, 5.0, 0.5)
    let mut app = test_app_with_registries();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should be spawned");

    let material_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("bolt should have MeshMaterial2d");

    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials
        .get(&material_handle.0)
        .expect("material should exist in Assets");

    let expected_color = Color::linear_rgb(6.0, 5.0, 0.5);
    assert_eq!(
        material.color, expected_color,
        "bolt material color should match definition color_rgb"
    );
}

// ── Behavior 3: spawn_bolt on subsequent node launches with random angle within BoltAngleSpread ──

#[test]
fn spawn_bolt_subsequent_node_launches_with_angle_within_spread() {
    // Given: node_index == 1, base_speed: 720.0
    // When: spawn_bolt runs
    // Then: velocity non-zero, upward, angle within DEFAULT_BOLT_ANGLE_SPREAD (0.524 rad)
    let mut app = test_app_with_registries();
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    assert!(
        vel.0.y > 0.0,
        "bolt should launch upward, got vy={}",
        vel.0.y
    );
    assert!(
        vel.speed() > 0.0,
        "bolt should have non-zero speed, got {}",
        vel.speed()
    );

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "launch angle {angle:.3} rad should be within DEFAULT_BOLT_ANGLE_SPREAD ({DEFAULT_BOLT_ANGLE_SPREAD:.3} rad)"
    );

    // Speed should be approximately 720.0 (definition base_speed)
    let speed = vel.speed();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "speed should be approximately 720.0 (from definition), got {speed:.1}"
    );
}

// ── Behavior 4: spawn_bolt uses definition radius for Scale2D and Aabb2D ──

#[test]
fn spawn_bolt_uses_definition_radius_for_scale_and_aabb() {
    // Given: BoltDefinition.radius: 14.0
    // When: spawn_bolt runs
    // Then: Scale2D { x: 14.0, y: 14.0 }, Aabb2D half_extents (14.0, 14.0), BoltRadius(14.0)
    use rantzsoft_physics2d::aabb::Aabb2D;
    use rantzsoft_spatial2d::components::Scale2D;

    let mut app = test_app_with_registries();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let world = app.world();

    let scale = world
        .get::<Scale2D>(entity)
        .expect("bolt should have Scale2D");
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON && (scale.y - 14.0).abs() < f32::EPSILON,
        "Scale2D should be (14.0, 14.0), got ({}, {})",
        scale.x,
        scale.y
    );

    let aabb = world
        .get::<Aabb2D>(entity)
        .expect("bolt should have Aabb2D");
    assert!(
        (aabb.half_extents.x - 14.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 14.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (14.0, 14.0), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y
    );

    let radius = world
        .get::<BoltRadius>(entity)
        .expect("bolt should have BoltRadius");
    assert!(
        (radius.0 - 14.0).abs() < f32::EPSILON,
        "BoltRadius should be 14.0, got {}",
        radius.0
    );
}

#[test]
fn spawn_bolt_uses_definition_radius_edge_case_small_radius() {
    // Edge case: BoltDefinition.radius: 6.0
    use rantzsoft_physics2d::aabb::Aabb2D;
    use rantzsoft_spatial2d::components::Scale2D;

    let mut app = test_app_with_registries();
    // Override the bolt definition with a smaller radius
    let small_def = BoltDefinition {
        name: "Bolt".to_string(),
        radius: 6.0,
        ..make_default_bolt_definition()
    };
    let mut bolt_registry = BoltRegistry::default();
    bolt_registry.insert("Bolt".to_string(), small_def);
    app.insert_resource(bolt_registry);

    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let world = app.world();
    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON && (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D should be (6.0, 6.0), got ({}, {})",
        scale.x,
        scale.y
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 6.0).abs() < f32::EPSILON
            && (aabb.half_extents.y - 6.0).abs() < f32::EPSILON,
        "Aabb2D half_extents should be (6.0, 6.0), got ({}, {})",
        aabb.half_extents.x,
        aabb.half_extents.y
    );
}

// ── Behavior 5: spawn_bolt uses definition speed values ──

#[test]
fn spawn_bolt_uses_definition_speed_values() {
    // Given: BoltDefinition base_speed: 720.0, min: 360.0, max: 1440.0. node_index == 1.
    // Then: BaseSpeed(720.0), MinSpeed(360.0), MaxSpeed(1440.0). Velocity magnitude ~ 720.0.
    use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed};

    let mut app = test_app_with_registries();
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let world = app.world();

    let base = world
        .get::<BaseSpeed>(entity)
        .expect("bolt should have BaseSpeed");
    assert!(
        (base.0 - 720.0).abs() < f32::EPSILON,
        "BaseSpeed should be 720.0, got {}",
        base.0
    );

    let min = world
        .get::<MinSpeed>(entity)
        .expect("bolt should have MinSpeed");
    assert!(
        (min.0 - 360.0).abs() < f32::EPSILON,
        "MinSpeed should be 360.0, got {}",
        min.0
    );

    let max = world
        .get::<MaxSpeed>(entity)
        .expect("bolt should have MaxSpeed");
    assert!(
        (max.0 - 1440.0).abs() < f32::EPSILON,
        "MaxSpeed should be 1440.0, got {}",
        max.0
    );

    let vel = world.get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.speed() - 720.0).abs() < 2.0,
        "velocity magnitude should be approximately 720.0, got {}",
        vel.speed()
    );
}

// ── Behavior 6: spawn_bolt reads from BoltRegistry ──

#[test]
fn spawn_bolt_uses_definition_values() {
    // Given: Registry has BoltDefinition base_speed: 720.0.
    //        Breaker at (0.0, -250.0). node_index == 1.
    // Then: Speed is 720.0 (from definition).
    //       Position Y is -196.0 (from DEFAULT_BOLT_SPAWN_OFFSET_Y 54.0).
    let mut app = test_app_with_registries();
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    let speed = vel.speed();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "speed should be 720.0 (from definition). Got {speed:.1}"
    );

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Position2D");

    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y; // -196.0
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "bolt y should be {expected_y} (from definition offset {DEFAULT_BOLT_SPAWN_OFFSET_Y}). Got {}",
        pos.0.y
    );
}

// ── Behavior 7: spawn_bolt sends BoltSpawned even when bolt already exists ──
// (Already covered by existing test, but verify with registry setup)

#[test]
fn existing_bolt_still_triggers_bolt_spawned_with_registries() {
    let mut app = test_app_with_registries();
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .primary()
        .spawn(app.world_mut());
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let messages = app.world().resource::<Messages<BoltSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_bolt must send BoltSpawned even when bolt already exists"
    );
}

// ── Behavior 8: spawn_bolt uses BreakerRegistry to look up bolt name from SelectedBreaker ──

#[test]
fn spawn_bolt_uses_breaker_registry_for_bolt_lookup() {
    // Given: SelectedBreaker("Chrono"). BreakerRegistry has "Chrono" with bolt: "HeavyBolt".
    //        BoltRegistry has "HeavyBolt" with base_speed: 500.0, radius: 20.0, base_damage: 25.0.
    // Then: Bolt has BoltDefinitionRef("HeavyBolt"), BoltBaseDamage(25.0), BoltRadius(20.0).
    let mut app = test_app_with_registries();

    let heavy_def = BoltDefinition {
        name: "HeavyBolt".to_string(),
        base_speed: 500.0,
        min_speed: 250.0,
        max_speed: 1000.0,
        radius: 20.0,
        base_damage: 25.0,
        effects: vec![],
        color_rgb: [1.0, 0.0, 0.0],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
    };
    app.world_mut()
        .resource_mut::<BoltRegistry>()
        .insert("HeavyBolt".to_string(), heavy_def);

    let chrono_def: BreakerDefinition =
        ron::de::from_str(r#"(name: "Chrono", bolt: "HeavyBolt", life_pool: None, effects: [])"#)
            .expect("test RON should parse");
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Chrono".to_string(), chrono_def);

    app.insert_resource(SelectedBreaker("Chrono".to_string()));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should be spawned");

    let world = app.world();

    let def_ref = world.get::<BoltDefinitionRef>(entity);
    assert!(def_ref.is_some(), "bolt should have BoltDefinitionRef");
    assert_eq!(
        def_ref.unwrap().0,
        "HeavyBolt",
        "BoltDefinitionRef should be 'HeavyBolt'"
    );

    let damage = world.get::<BoltBaseDamage>(entity);
    assert!(damage.is_some(), "bolt should have BoltBaseDamage");
    assert!(
        (damage.unwrap().0 - 25.0).abs() < f32::EPSILON,
        "BoltBaseDamage should be 25.0, got {}",
        damage.unwrap().0
    );

    let radius = world.get::<BoltRadius>(entity);
    assert!(radius.is_some(), "bolt should have BoltRadius");
    assert!(
        (radius.unwrap().0 - 20.0).abs() < f32::EPSILON,
        "BoltRadius should be 20.0, got {}",
        radius.unwrap().0
    );
}

// ── Behavior 9: spawn_bolt uses BoltSpawnOffsetY from definition for spawn position ──

#[test]
fn spawn_bolt_uses_spawn_offset_y_with_breaker_entity() {
    // Given: Breaker at (50.0, -100.0). DEFAULT_BOLT_SPAWN_OFFSET_Y = 54.0.
    // Then: Position = (50.0, -100.0 + 54.0) = (50.0, -46.0).
    let mut app = test_app_with_registries();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(50.0, -100.0)),
        rantzsoft_spatial2d::components::Spatial2D,
        GameDrawLayer::Breaker,
    ));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Position2D");

    let expected = Vec2::new(50.0, -100.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0.x - expected.x).abs() < f32::EPSILON && (pos.0.y - expected.y).abs() < f32::EPSILON,
        "bolt position should be {expected:?}, got {:?}",
        pos.0
    );
}

#[test]
fn spawn_bolt_uses_spawn_offset_y_without_breaker_entity() {
    // Edge case: No breaker entity. Falls back to BreakerDefinition.y_position (-250.0).
    // Position = (0.0, -250.0 + 54.0) = (0.0, -196.0).
    let mut app = test_app_with_registries();
    app.add_systems(Startup, spawn_bolt);
    app.update();

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Position2D");

    let expected_y = BreakerDefinition::default().y_position + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "bolt y should be {expected_y} (BreakerDefinition.y_position + offset), got {}",
        pos.0.y
    );
}

// ── Behavior 10: spawn_bolt returns early on unknown breaker name ──

#[test]
fn spawn_bolt_returns_early_on_unknown_breaker_name() {
    let mut app = test_app_with_registries();
    app.insert_resource(SelectedBreaker("Unknown".to_string()));
    app.add_systems(Startup, spawn_bolt);
    app.update();

    // No bolt should be spawned — system returns early with a warning
    let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(count, 0, "no bolt should spawn for unknown breaker");
}
