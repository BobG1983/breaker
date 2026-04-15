//! Tests for error handling, edge cases, combined behavior, and lookup chain
//! (behaviors 24-27).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::{make_aegis_breaker_definition, test_app};
use crate::{
    bolt::{definition::BoltDefinition, messages::BoltSpawned, registry::BoltRegistry},
    breaker::{
        definition::BreakerDefinition, messages::BreakerSpawned, registry::BreakerRegistry,
        resources::SelectedBreaker,
    },
    prelude::*,
};

// ════════════════════════════════════════════════════════════════════════
// Behavior 24: Selected breaker not in registry
// ════════════════════════════════════════════════════════════════════════

#[test]
fn setup_run_does_not_panic_when_selected_breaker_missing() {
    let mut app = test_app();
    app.insert_resource(SelectedBreaker("NonExistent".to_string()));
    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 0,
        "no breaker should be spawned when selected breaker is not in registry"
    );

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 0,
        "no bolt should be spawned when selected breaker is not in registry"
    );

    let breaker_msgs = app.world().resource::<Messages<BreakerSpawned>>();
    assert_eq!(
        breaker_msgs.iter_current_update_messages().count(),
        0,
        "no BreakerSpawned message when selected breaker missing from registry"
    );

    let bolt_msgs = app.world().resource::<Messages<BoltSpawned>>();
    assert_eq!(
        bolt_msgs.iter_current_update_messages().count(),
        0,
        "no BoltSpawned message when selected breaker missing from registry"
    );
}

#[test]
fn setup_run_does_not_panic_with_empty_breaker_registry() {
    // Edge case: empty registry
    let mut app = test_app();
    app.world_mut().resource_mut::<BreakerRegistry>().clear();
    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 0,
        "no breaker should be spawned with empty breaker registry"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 25: Bolt name from breaker definition not in BoltRegistry
// ════════════════════════════════════════════════════════════════════════

#[test]
fn setup_run_spawns_breaker_but_not_bolt_when_bolt_missing_from_registry() {
    let mut app = test_app();
    // Override the Aegis definition to reference a bolt that doesn't exist
    let mut aegis_def = make_aegis_breaker_definition();
    aegis_def.bolt = "MissingBolt".to_string();
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("Aegis".to_string(), aegis_def);

    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 1,
        "breaker should still be spawned even when bolt is missing from registry"
    );

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 0,
        "bolt should NOT be spawned when bolt name is not in BoltRegistry"
    );

    let breaker_msgs = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        breaker_msgs.iter_current_update_messages().count() > 0,
        "BreakerSpawned message should still be sent"
    );

    let bolt_msgs = app.world().resource::<Messages<BoltSpawned>>();
    assert_eq!(
        bolt_msgs.iter_current_update_messages().count(),
        0,
        "no BoltSpawned when bolt name missing from registry"
    );
}

#[test]
fn setup_run_spawns_breaker_but_not_bolt_with_empty_bolt_registry() {
    // Edge case: empty BoltRegistry
    let mut app = test_app();
    app.world_mut().resource_mut::<BoltRegistry>().clear();

    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 1,
        "breaker should be spawned even with empty bolt registry"
    );

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 0,
        "bolt should NOT be spawned with empty bolt registry"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 26: Both breaker and bolt in a single invocation
// ════════════════════════════════════════════════════════════════════════

#[test]
fn setup_run_spawns_both_breaker_and_bolt_in_single_invocation() {
    let mut app = test_app();
    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(breaker_count, 1, "exactly 1 breaker should exist");

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(bolt_count, 1, "exactly 1 bolt should exist");

    let breaker_msgs = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        breaker_msgs.iter_current_update_messages().count() > 0,
        "BreakerSpawned should be sent"
    );

    let bolt_msgs = app.world().resource::<Messages<BoltSpawned>>();
    assert!(
        bolt_msgs.iter_current_update_messages().count() > 0,
        "BoltSpawned should be sent"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Behavior 27: Bolt definition lookup chain
// ════════════════════════════════════════════════════════════════════════

#[test]
fn setup_run_reads_bolt_name_from_breaker_definition() {
    // SelectedBreaker("TestBreaker") -> BreakerDefinition.bolt == "HeavyBolt"
    // -> BoltRegistry["HeavyBolt"] with base_speed: 500.0
    let mut app = test_app();

    let test_breaker_def: BreakerDefinition = ron::de::from_str(
        r#"(name: "TestBreaker", bolt: "HeavyBolt", life_pool: Some(3), effects: [])"#,
    )
    .expect("test RON should parse");
    app.world_mut()
        .resource_mut::<BreakerRegistry>()
        .insert("TestBreaker".to_string(), test_breaker_def);

    let heavy_bolt_def = BoltDefinition {
        name:                 "HeavyBolt".to_string(),
        base_speed:           500.0,
        min_speed:            250.0,
        max_speed:            1000.0,
        radius:               20.0,
        base_damage:          25.0,
        effects:              vec![],
        color_rgb:            [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical:   5.0,
        min_radius:           None,
        max_radius:           None,
    };
    app.world_mut()
        .resource_mut::<BoltRegistry>()
        .insert("HeavyBolt".to_string(), heavy_bolt_def);

    app.insert_resource(SelectedBreaker("TestBreaker".to_string()));

    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist");

    let base_speed = app
        .world()
        .get::<BaseSpeed>(entity)
        .expect("bolt should have BaseSpeed");
    assert!(
        (base_speed.0 - 500.0).abs() < f32::EPSILON,
        "BaseSpeed should be 500.0 (from HeavyBolt, not default Bolt), got {}",
        base_speed.0
    );
}
